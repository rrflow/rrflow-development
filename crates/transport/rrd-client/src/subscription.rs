//! Current durable-subscription socket pending B-04 multiplexing.

use crate::error::{contract, decode, websocket_connect_error};
use crate::{Error, RequestOptions, Result, RrdClient, Session};
use futures_util::{SinkExt, StreamExt};
use hyper::Method;
use rrd_contract::{
    CloseSubscription, CloseSubscriptionResult, CorrelationId, OpenSubscription,
    OpenSubscriptionResult, SubscriptionClientFrame, SubscriptionServerFrame, SubscriptionSnapshot,
};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub struct SubscriptionSocket {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    subscription_id: CorrelationId,
    connection_generation: u64,
}

impl RrdClient {
    pub async fn open_subscription(
        &self,
        session: &Session,
        request: OpenSubscription,
        options: RequestOptions,
    ) -> Result<OpenSubscriptionResult> {
        self.session_call(
            Method::POST,
            "/v1/subscriptions/open",
            session,
            request,
            options,
            true,
        )
        .await
    }

    pub async fn close_subscription(
        &self,
        session: &Session,
        request: CloseSubscription,
        options: RequestOptions,
    ) -> Result<CloseSubscriptionResult> {
        self.session_call(
            Method::POST,
            "/v1/subscriptions/close",
            session,
            request,
            options,
            true,
        )
        .await
    }

    /// Connects or reconnects the durable subscription. The server increments
    /// its fencing generation and replays from the last durably acknowledged
    /// runtime cursor.
    pub async fn connect_subscription(
        &self,
        session: &Session,
        subscription_id: CorrelationId,
    ) -> Result<(SubscriptionSocket, SubscriptionSnapshot)> {
        let origin = if let Some(rest) = self.endpoint.strip_prefix("http://") {
            format!("ws://{rest}")
        } else if let Some(rest) = self.endpoint.strip_prefix("https://") {
            format!("wss://{rest}")
        } else {
            return Err(Error::Contract(
                "RRD endpoint has no WebSocket scheme".into(),
            ));
        };
        let url = format!(
            "{origin}/v1/subscriptions/{}/stream",
            subscription_id.as_str()
        );
        let mut request = url.into_client_request().map_err(contract)?;
        request.headers_mut().insert(
            "x-rrd-session",
            session.session_id().as_str().parse().map_err(contract)?,
        );
        request.headers_mut().insert(
            "authorization",
            format!("Bearer {}", session.token().as_str())
                .parse()
                .map_err(contract)?,
        );
        let connector = match &self.websocket_tls {
            Some(tls) => tokio_tungstenite::Connector::Rustls(tls.clone()),
            None => tokio_tungstenite::Connector::Plain,
        };
        let connected = tokio::time::timeout(
            self.config.request_timeout,
            tokio_tungstenite::connect_async_tls_with_config(request, None, false, Some(connector)),
        )
        .await
        .map_err(|_| Error::Timeout)?
        .map_err(websocket_connect_error)?;
        let mut socket = SubscriptionSocket {
            stream: connected.0,
            subscription_id,
            connection_generation: 0,
        };
        let opened = socket.receive().await?;
        match opened {
            SubscriptionServerFrame::Opened { subscription }
                if subscription.subscription_id == socket.subscription_id
                    && subscription.connection_generation > 0 =>
            {
                socket.connection_generation = subscription.connection_generation;
                Ok((socket, subscription))
            }
            SubscriptionServerFrame::Error {
                error,
                acknowledged_cursor,
            } => Err(Error::Subscription {
                error,
                acknowledged_cursor,
            }),
            _ => Err(Error::Decode(
                "subscription WebSocket did not begin with an opened frame".into(),
            )),
        }
    }
}

impl SubscriptionSocket {
    pub fn subscription_id(&self) -> &CorrelationId {
        &self.subscription_id
    }

    pub fn connection_generation(&self) -> u64 {
        self.connection_generation
    }

    pub async fn receive(&mut self) -> Result<SubscriptionServerFrame> {
        loop {
            match self.stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    let frame =
                        serde_json::from_str::<SubscriptionServerFrame>(&text).map_err(decode)?;
                    if let SubscriptionServerFrame::Opened { subscription }
                    | SubscriptionServerFrame::Acknowledged { subscription } = &frame
                    {
                        if subscription.subscription_id != self.subscription_id {
                            return Err(Error::ResponseIdentityMismatch);
                        }
                        self.connection_generation = subscription.connection_generation;
                    }
                    if let SubscriptionServerFrame::Error {
                        error,
                        acknowledged_cursor,
                    } = &frame
                    {
                        return Err(Error::Subscription {
                            error: error.clone(),
                            acknowledged_cursor: *acknowledged_cursor,
                        });
                    }
                    return Ok(frame);
                }
                Some(Ok(Message::Ping(bytes))) => self
                    .stream
                    .send(Message::Pong(bytes))
                    .await
                    .map_err(|error| Error::Transport(error.to_string()))?,
                Some(Ok(Message::Pong(_))) => {}
                Some(Ok(Message::Binary(_))) => {
                    return Err(Error::Decode(
                        "subscription server sent an unsupported binary frame".into(),
                    ));
                }
                Some(Ok(Message::Close(frame))) => {
                    return Err(Error::Transport(format!(
                        "subscription WebSocket closed: {frame:?}"
                    )));
                }
                Some(Err(error)) => return Err(Error::Transport(error.to_string())),
                None => return Err(Error::Transport("subscription WebSocket ended".into())),
                Some(Ok(Message::Frame(_))) => {}
            }
        }
    }

    pub async fn acknowledge(&mut self, delivery_sequence: u64, through_cursor: u64) -> Result<()> {
        self.send(SubscriptionClientFrame::Ack {
            connection_generation: self.connection_generation,
            delivery_sequence,
            through_cursor,
        })
        .await
    }

    pub async fn heartbeat(&mut self) -> Result<()> {
        self.send(SubscriptionClientFrame::Heartbeat {
            connection_generation: self.connection_generation,
        })
        .await
    }

    pub async fn close(&mut self) -> Result<()> {
        self.send(SubscriptionClientFrame::Close {
            connection_generation: self.connection_generation,
        })
        .await
    }

    async fn send(&mut self, frame: SubscriptionClientFrame) -> Result<()> {
        let text = serde_json::to_string(&frame).map_err(decode)?;
        self.stream
            .send(Message::Text(text.into()))
            .await
            .map_err(|error| Error::Transport(error.to_string()))
    }
}
