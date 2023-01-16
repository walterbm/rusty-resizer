use std::{
    collections::HashSet,
    future::{ready, Ready},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Instant,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use cadence::{StatsdClient, Timed};
use futures_util::Future;
use pin_project_lite::pin_project;

/// Factory to create a StatsDMiddleware that can be used to emit basic request metrics using StatsD.
pub struct StatsD {
    client: Arc<StatsdClient>,
    exclude: HashSet<String>,
}

impl StatsD {
    pub fn new(client: Arc<StatsdClient>) -> Self {
        Self {
            client,
            exclude: HashSet::new(),
        }
    }

    pub fn exclude<T: Into<String>>(mut self, path: T) -> Self {
        self.exclude.insert(path.into());
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for StatsD
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = StatsDMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(StatsDMiddleware {
            service,
            client: self.client.clone(),
            exclude: self.exclude.clone(),
        }))
    }
}

/// Middleware that will automatically emit StatsD basic metrics for every request. Currently tracks:
/// * Status Code
/// * Response Time
///
/// Specific urls can be explicitly excluded to avoid collecting unnecessary metrics.
pub struct StatsDMiddleware<S> {
    service: S,
    client: Arc<StatsdClient>,
    exclude: HashSet<String>,
}

impl<S, B> Service<ServiceRequest> for StatsDMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = StatsDFuture<S::Future>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let statsd = self.client.clone();
        let exclude = self.exclude.contains(req.path());
        // return a custom future to avoid relying on BoxFuture and adding extra allocations
        StatsDFuture {
            start,
            statsd,
            exclude,
            future: self.service.call(req),
        }
    }
}

// pin project is used to access underlying future
pin_project! {
    pub struct StatsDFuture<F> {
        #[pin]
        future: F,
        start: Instant,
        exclude: bool,
        statsd: Arc<StatsdClient>,
    }
}

impl<F, B> Future for StatsDFuture<F>
where
    F: Future<Output = Result<ServiceResponse<B>, Error>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if *this.exclude {
            return this.future.poll(cx);
        }

        let result = match this.future.poll(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(result) => result,
        };

        match result {
            Ok(res) => {
                let tag = &res.request().path()[1..].replace('/', ".");
                this.statsd
                    .time_with_tags(tag, this.start.elapsed())
                    .with_tag("status", res.response().status().as_str())
                    .try_send()
                    .ok();
                Poll::Ready(Ok(res))
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_http::StatusCode;
    use actix_service::{IntoService, Service, Transform};
    use actix_web::{test::TestRequest, HttpResponse};
    use cadence::SpyMetricSink;
    use futures_util::future::ok;

    use super::*;

    #[actix_rt::test]
    async fn test_statsd_emit_metrics() {
        let (rx, sink) = SpyMetricSink::new();

        let srv = |req: ServiceRequest| {
            ok(req.into_response(HttpResponse::build(StatusCode::OK).finish()))
        };
        let statsd = StatsD::new(Arc::new(StatsdClient::from_sink("testing", sink)));

        let srv = statsd.new_transform(srv.into_service()).await.unwrap();

        let req = TestRequest::default().uri("/test/track").to_srv_request();
        let _res = srv.call(req).await.unwrap();
        let sent = rx.try_recv().unwrap();
        assert_eq!(
            "testing.test.track:0|ms|#status:200",
            std::str::from_utf8(sent.as_slice()).unwrap()
        );
    }

    #[actix_rt::test]
    async fn test_statsd_skip_metrics_for_excluded_paths() {
        let (rx, sink) = SpyMetricSink::new();

        let srv = |req: ServiceRequest| {
            ok(req.into_response(HttpResponse::build(StatusCode::OK).finish()))
        };
        let statsd = StatsD::new(Arc::new(StatsdClient::from_sink("testing", sink)))
            .exclude("/test/exclude");

        let srv = statsd.new_transform(srv.into_service()).await.unwrap();

        let req = TestRequest::default().uri("/test/exclude").to_srv_request();
        let _res = srv.call(req).await.unwrap();
        assert!(rx.try_recv().is_err());
    }
}
