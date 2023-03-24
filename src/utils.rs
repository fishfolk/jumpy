use async_channel::*;

/// Create a bi-directional channel with a given request and response type.
pub fn bi_channel<Request, Response>() -> (
    BiChannelClient<Request, Response>,
    BiChannelServer<Request, Response>,
) {
    let (request_sender, request_receiver) = async_channel::unbounded();
    let (response_sender, response_receiver) = async_channel::unbounded();

    (
        BiChannelClient {
            request_sender,
            response_receiver,
        },
        BiChannelServer {
            request_receiver,
            response_sender,
        },
    )
}

/// A [`bi_channel`] client.
pub struct BiChannelClient<Request, Response> {
    pub request_sender: Sender<Request>,
    pub response_receiver: Receiver<Response>,
}
impl<Req, Res> BiChannelClient<Req, Res> {
    pub fn recv_blocking(&self) -> Result<Res, RecvError> {
        self.response_receiver.recv_blocking()
    }
    pub fn send_blocking(&self, req: Req) -> Result<(), SendError<Req>> {
        self.request_sender.send_blocking(req)
    }
    pub async fn send(&self, req: Req) -> Result<(), SendError<Req>> {
        self.request_sender.send(req).await
    }
    pub async fn recv(&self) -> Result<Res, RecvError> {
        self.response_receiver.recv().await
    }
    pub fn try_send(&self, req: Req) -> Result<(), TrySendError<Req>> {
        self.request_sender.try_send(req)
    }
    pub fn try_recv(&self) -> Result<Res, TryRecvError> {
        self.response_receiver.try_recv()
    }
}

/// A [`bi_channel`] server.
pub struct BiChannelServer<Request, Response> {
    pub request_receiver: Receiver<Request>,
    pub response_sender: Sender<Response>,
}
impl<Req, Res> BiChannelServer<Req, Res> {
    pub fn recv_blocking(&self) -> Result<Req, RecvError> {
        self.request_receiver.recv_blocking()
    }
    pub fn send_blocking(&self, res: Res) -> Result<(), SendError<Res>> {
        self.response_sender.send_blocking(res)
    }
    pub async fn send(&self, res: Res) -> Result<(), SendError<Res>> {
        self.response_sender.send(res).await
    }
    pub async fn recv(&self) -> Result<Req, RecvError> {
        self.request_receiver.recv().await
    }
    pub fn try_send(&self, res: Res) -> Result<(), TrySendError<Res>> {
        self.response_sender.try_send(res)
    }
    pub fn try_recv(&self) -> Result<Req, TryRecvError> {
        self.request_receiver.try_recv()
    }
}
