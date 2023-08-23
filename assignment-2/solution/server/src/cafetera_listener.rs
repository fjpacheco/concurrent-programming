use std::net::SocketAddr;

use crate::nodo::ReceiveFromCafetera;
use crate::utils::{id_to_addr_read_data, id_to_addr_write_data, MAX_UDP_SIZE};
use crate::{
    error_server::{ErrorServer, TipoError},
    nodo::Nodo,
};
use actix::fut::wrap_future;
use actix::{Actor, ActorFutureExt, ContextFutureSpawner, Message};

use actix::{Addr, Context, Handler};
use tokio::net::UdpSocket;
pub struct CafeteraListener {
    /// direccion del mail box del actor Nodo
    addr_actor_nodo: Addr<Nodo>,
    /// Socket udp donde puede recibir mensajes de la/s cafetera
    socket_nodo_to_write: Option<UdpSocket>,
}

/// Actor encargado de recibir y enviar mensajes a la cafetera por udp.
/// Tambien se comunica con otro actor para intercambiar mensajes.
impl Actor for CafeteraListener {
    type Context = Context<Self>;
}

impl CafeteraListener {
    /// Crea los sockets, empieza un nuevo actor CafeteraListener y dispara una Task
    /// encargada de escuchar por udp los mensajes provenientes de las cafeteras y enviarselos al actor
    pub async fn start(
        id_nodo: u8,
        addr_actor_nodo: Addr<Nodo>,
    ) -> Result<Addr<CafeteraListener>, ErrorServer> {
        let socket_nodo_to_read = UdpSocket::bind(id_to_addr_read_data(id_nodo))
            .await
            .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorConexion))?;
        let socket_nodo_to_write = UdpSocket::bind(id_to_addr_write_data(id_nodo))
            .await
            .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorConexion))?;

        let addr_actor = CafeteraListener {
            addr_actor_nodo,
            socket_nodo_to_write: Some(socket_nodo_to_write),
        }
        .start();
        let addr_actor_clone = addr_actor.clone();
        // invocar una task con tokio que escuche los mensajes y los envie al actor
        tokio::spawn(async move {
            let mut buf: [u8; MAX_UDP_SIZE] = [0; MAX_UDP_SIZE];
            loop {
                let (cantidad_leida, socket) = socket_nodo_to_read
                    .recv_from(&mut buf)
                    .await
                    .expect("failed to receive from socket");
                addr_actor_clone.do_send(StreamHandlerUdp {
                    vec: buf[..cantidad_leida].to_vec(),
                    socket,
                });
            }
        });

        Ok(addr_actor)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
struct StreamHandlerUdp {
    /// Mensaje a pasar al metodo del actor nodo
    vec: Vec<u8>,
    /// Socket udp a pasar al método del actor nodo
    socket: SocketAddr,
}

/// Cuando recibe algo de la cafeterea lo forwardea al actor nodo con el socket_addr correspondiente
impl Handler<StreamHandlerUdp> for CafeteraListener {
    type Result = ();

    fn handle(&mut self, msg: StreamHandlerUdp, _ctx: &mut Context<Self>) -> Self::Result {
        self.addr_actor_nodo.do_send(ReceiveFromCafetera {
            msg: msg.vec,
            socket: msg.socket,
        });
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct ReceiverActorNodo {
    /// Mensaje a enviar
    pub vec: Vec<u8>,
    /// Socket udp a utilizar para el envio
    pub socket: SocketAddr,
}

/// Envia el mensaje vec a la cafetera con el socket udp correspondiente
impl Handler<ReceiverActorNodo> for CafeteraListener {
    type Result = ();

    fn handle(&mut self, msg: ReceiverActorNodo, ctx: &mut Context<Self>) -> Self::Result {
        let write = self
            .socket_nodo_to_write
            .take()
            .expect("No debería poder llegar otro mensaje antes de que vuelva por usar ctx.wait");
        wrap_future::<_, Self>(async move {
            write
                .send_to(&msg.vec, msg.socket)
                .await
                .expect("should have sent");
            write
        })
        .map(|write, this, _| this.socket_nodo_to_write = Some(write))
        .wait(ctx);
    }
}
