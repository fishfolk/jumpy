#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/87333478?s=200&v=4")]
// This cfg_attr is needed because `rustdoc::all` includes lints not supported on stable
#![cfg_attr(doc, allow(unknown_lints))]
#![deny(rustdoc::all)]

use std::{
    io,
    ops::Deref,
    sync::Arc,
    task::{ready, Context, Poll},
};

use async_io::Async;
use bevy_tasks::IoTaskPool;
use quinn_proto::Transmit;

#[derive(Clone, Debug)]
pub struct BevyIoTaskPoolExecutor;

#[derive(Clone, Debug)]
pub struct AsyncExecutor<'a>(pub Arc<async_executor::Executor<'a>>);

impl<'a> Deref for AsyncExecutor<'a> {
    type Target = async_executor::Executor<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
#[pin_project::pin_project]
pub struct AsyncIoTimer(#[pin] pub async_io::Timer);

impl quinn::AsyncTimer for AsyncIoTimer {
    fn reset(mut self: std::pin::Pin<&mut Self>, i: std::time::Instant) {
        self.0 = async_io::Timer::at(i);
    }

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let pinned_timer = self.project().0;
        <async_io::Timer as std::future::Future>::poll(pinned_timer, cx).map(|_| ())
    }
}

impl quinn::Runtime for AsyncExecutor<'static> {
    fn new_timer(&self, i: std::time::Instant) -> std::pin::Pin<Box<dyn quinn::AsyncTimer>> {
        Box::pin(AsyncIoTimer(async_io::Timer::at(i))) as _
    }

    fn spawn(&self, future: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>) {
        self.0.spawn(future).detach();
    }

    fn wrap_udp_socket(
        &self,
        sock: std::net::UdpSocket,
    ) -> io::Result<Box<dyn quinn::AsyncUdpSocket>> {
        quinn_udp::UdpSocketState::configure((&sock).into())?;
        Ok(Box::new(AsyncIoUdpSocket {
            io: Async::new(sock)?,
            inner: quinn_udp::UdpSocketState::new(),
        }))
    }
}

impl quinn::Runtime for BevyIoTaskPoolExecutor {
    fn new_timer(&self, i: std::time::Instant) -> std::pin::Pin<Box<dyn quinn::AsyncTimer>> {
        Box::pin(AsyncIoTimer(async_io::Timer::at(i))) as _
    }

    fn spawn(&self, future: std::pin::Pin<Box<dyn futures_lite::Future<Output = ()> + Send>>) {
        let io_pool = IoTaskPool::get();

        io_pool.spawn(future).detach();
    }

    fn wrap_udp_socket(
        &self,
        sock: std::net::UdpSocket,
    ) -> io::Result<Box<dyn quinn::AsyncUdpSocket>> {
        Ok(Box::new(AsyncIoUdpSocket {
            io: Async::new(sock)?,
            inner: quinn_udp::UdpSocketState::new(),
        }))
    }
}

#[derive(Debug)]
struct AsyncIoUdpSocket {
    io: async_io::Async<std::net::UdpSocket>,
    inner: quinn_udp::UdpSocketState,
}

impl quinn::AsyncUdpSocket for AsyncIoUdpSocket {
    fn poll_send(
        &mut self,
        state: &quinn_udp::UdpState,
        cx: &mut Context,
        transmits: &[Transmit],
    ) -> Poll<io::Result<usize>> {
        loop {
            ready!(self.io.poll_writable(cx))?;
            if let Ok(res) = self.inner.send((&self.io).into(), state, transmits) {
                return Poll::Ready(Ok(res));
            }
        }
    }

    fn poll_recv(
        &self,
        cx: &mut Context,
        bufs: &mut [io::IoSliceMut<'_>],
        meta: &mut [quinn_udp::RecvMeta],
    ) -> Poll<io::Result<usize>> {
        loop {
            ready!(self.io.poll_readable(cx))?;
            if let Ok(res) = self.inner.recv((&self.io).into(), bufs, meta) {
                return Poll::Ready(Ok(res));
            }
        }
    }

    fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.io.as_ref().local_addr()
    }
}
