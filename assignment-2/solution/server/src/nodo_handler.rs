use std::net::SocketAddr;
use std::sync::Arc;

use actix::fut::wrap_future;
use actix::{
    Actor, ActorContext, Addr, Context, ContextFutureSpawner, Handler, Message, StreamHandler,
};

use crate::coordinador::{Coordinador, Disconnect, DisconnectNodo};
use crate::mensaje::{
    Abort, Finish, Mensaje, MensajeBytes, OkeyAbortToCoordinator, OkeyToCoordinator, PingCord,
    Starter, Yes,
};
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct NodoHandler {
    pub addr_coordinador: Addr<Coordinador>,
    pub write: Arc<Mutex<WriteHalf<TcpStream>>>,
    pub addr: SocketAddr,
    pub id_nodo: u8,
    pub conectado: bool,
}

impl Actor for NodoHandler {
    type Context = Context<Self>;
}

impl StreamHandler<Result<String, std::io::Error>> for NodoHandler {
    fn handle(&mut self, read: Result<String, std::io::Error>, _ctx: &mut Self::Context) {
        if let Ok(line) = read {
            let aux = line.bytes().collect::<Vec<u8>>();
            let tipo_mensaje = Mensaje::from_bytes(aux[0]);
            let _arc = self.write.clone();
            let addr_coor_clone = self.addr_coordinador.clone();

            match tipo_mensaje {
                Mensaje::STARTER => {
                    let mensaje = Starter::from_string(line);

                    addr_coor_clone.do_send(mensaje);
                }
                Mensaje::YES => {
                    let mensaje = Yes::from_string(line);
                    addr_coor_clone.do_send(mensaje);
                }
                Mensaje::PING => {
                    let mensaje = PingCord::from_string(line);
                    addr_coor_clone.do_send(mensaje);
                }
                Mensaje::FINISH => {
                    let mensaje = Finish::from_string(line);
                    addr_coor_clone.do_send(mensaje);
                }
                Mensaje::OKEY => {
                    let mensaje = OkeyToCoordinator::from_string(line);
                    addr_coor_clone.do_send(mensaje);
                }
                Mensaje::OKEYABORT => {
                    let mensaje = OkeyAbortToCoordinator::from_string(line);
                    addr_coor_clone.do_send(mensaje);
                }
                Mensaje::ABORT => {
                    let mensaje = Abort::from_string(line);
                    addr_coor_clone.do_send(mensaje);
                }
                // TODO: Mensaje Disconnect, nunca recibo un prepare entonces funciona
                Mensaje::PREPARE => {
                    self.conectado = false;
                    let mensaje = Disconnect {
                        tipo_mensaje: Mensaje::DISCONNECT as u8,
                    };
                    addr_coor_clone.do_send(mensaje);
                }
                _ => println!("[NODO-{}] [HANDLER-COORDINADOR] MSG NO RECONOCIDO", aux[1]),
            };
        } else {
        }
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        if self.conectado {
            if let Err(err) = self.addr_coordinador.try_send(DisconnectNodo {
                id_nodo: self.id_nodo,
            }) {
                println!(
                    "[HANDLER-COORDINADOR] Error al enviar DISCONNECT al coordinador {:?}",
                    err
                );
            }
        }
        ctx.stop();
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct ReceiverFromCoordinador {
    pub string: String,
}

impl Handler<ReceiverFromCoordinador> for NodoHandler {
    type Result = ();
    fn handle(
        &mut self,
        mut msg: ReceiverFromCoordinador,
        ctx: &mut Context<Self>,
    ) -> Self::Result {
        let arc = self.write.clone();
        msg.string.push('\n');
        let id_nodo = self.id_nodo;
        wrap_future::<_, Self>(async move {
            let res = arc.lock().await.write_all(msg.string.as_bytes()).await;

            if let Err(e) = res {
                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    println!("El coordinador se cayó {:}", id_nodo);
                }
            };
        })
        .spawn(ctx);
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Shutdown;

impl Handler<Shutdown> for NodoHandler {
    type Result = ();
    fn handle(&mut self, _msg: Shutdown, ctx: &mut Context<Self>) -> Self::Result {
        let arc = self.write.clone();
        wrap_future::<_, Self>(async move {
            arc.lock()
                .await
                .shutdown()
                .await
                .expect("deberia hacer shutdown")
        })
        .spawn(ctx);
    }
}
