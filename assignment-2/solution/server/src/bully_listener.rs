use std::net::SocketAddr;
use std::time::Duration;

use crate::bully_messages::{
    Coordinator, Election, MensajeBully, MensajeBullyBytes, OkeyBully, Ping, PingCord,
};
use crate::nodo::ReceiveNewCoordinator;
use crate::utils::{
    id_to_addr_read_bully, id_to_addr_write_bully, CANT_MAX_NODOS, ID_CORDINADOR_INICIAL,
    MAX_UDP_SIZE, TIMEOUT_OK_BULLY_MILLIS,
};
use crate::{
    error_server::{ErrorServer, TipoError},
    nodo::Nodo,
};

use actix::clock::sleep;

use actix::fut::wrap_future;
use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, Message,
    ResponseActFuture, WrapFuture,
};
use tokio::net::UdpSocket;

/// Enum que representa los posibles estados del actor
#[derive(Debug, Clone, Copy)]
pub enum BullyState {
    NotExecuting = 0,
    WaitingOkey,
    WaitingCoordinator,
}

/// Estructura que almacena lo necesario para el actor bully listener
pub struct BullyListener {
    /// Dirección del actor nodo asociado a este bully listener
    addr_actor_nodo: Addr<Nodo>,
    /// Estado del actor
    estado: BullyState,
    /// Representa si el nodo esta conectado o no a la red
    conectado: bool,
    /// Socket utilizado para la escritura a otros bully listeners
    socket_nodo_to_write: Option<UdpSocket>,
    /// Id del nodo
    id_nodo: u8,
    /// Flag que indica si el nodo asociado es coordinador
    soy_coordinador: bool,
}

impl Actor for BullyListener {
    type Context = Context<Self>;
}

impl BullyListener {
    // asyn
    pub async fn start(
        id_nodo: u8,
        addr_actor_nodo: Addr<Nodo>,
    ) -> Result<Addr<BullyListener>, ErrorServer> {
        let socket_nodo_to_read = UdpSocket::bind(id_to_addr_write_bully(id_nodo))
            .await
            .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorConexion))?;

        let socket_nodo_to_write = UdpSocket::bind(id_to_addr_read_bully(id_nodo))
            .await
            .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorConexion))?;

        let addr_actor = BullyListener {
            addr_actor_nodo,
            estado: BullyState::NotExecuting,
            conectado: true,
            socket_nodo_to_write: Some(socket_nodo_to_write),
            id_nodo,
            soy_coordinador: id_nodo == ID_CORDINADOR_INICIAL,
        }
        .start();
        let addr_actor_bully_clone = addr_actor.clone();

        tokio::spawn(async move {
            let mut buf: [u8; MAX_UDP_SIZE] = [0; MAX_UDP_SIZE];
            loop {
                let (cantidad_leida, socket) = socket_nodo_to_read
                    .recv_from(&mut buf)
                    .await
                    .expect("failed to receive from socket");
                addr_actor_bully_clone.do_send(StreamHandlerUdp {
                    vec: buf[..cantidad_leida].to_vec(),
                    _socket: socket,
                });
            }
        });

        Ok(addr_actor)
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
struct StreamHandlerUdp {
    vec: Vec<u8>,
    _socket: SocketAddr,
}

/// Mensaje que se recibe cuando llega algo via UDP
/// El handler construye el mensaje y lo fordwardea según corresponda
impl Handler<StreamHandlerUdp> for BullyListener {
    type Result = ();

    fn handle(&mut self, msg: StreamHandlerUdp, ctx: &mut Context<Self>) -> Self::Result {
        if !self.conectado {
            return;
        }
        let tipo_mensaje = MensajeBully::from_bytes(msg.vec[0]);
        match tipo_mensaje {
            MensajeBully::OKEY => {
                let mensaje = OkeyBully::from_bytes(msg.vec);
                ctx.address().do_send(mensaje);
            }
            MensajeBully::ELECTION => {
                let mensaje = Election::from_bytes(msg.vec);
                ctx.address().do_send(mensaje);
            }
            MensajeBully::COORDINATOR => {
                let mensaje = Coordinator::from_bytes(msg.vec);
                ctx.address().do_send(mensaje);
            }
            MensajeBully::PING => {
                let mensaje = Ping::from_bytes(msg.vec);
                ctx.address().do_send(mensaje);
            }
            MensajeBully::PINGCORD => {
                let mensaje = PingCord::from_bytes(msg.vec);
                ctx.address().do_send(mensaje);
            }
            _ => println!("Mensaje desconocido"),
        }
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
struct TimeoutHandler;

/// Mensaje que se recibe cuando el future con un timeout creado al iniciar un election
/// termina. En la función se chequea si llego un Okey, en caso contrario el bully listener
/// se autoproclama coordinador y envia coordinator al resto de los nodos
impl Handler<TimeoutHandler> for BullyListener {
    type Result = ();

    fn handle(&mut self, _: TimeoutHandler, ctx: &mut Context<Self>) -> Self::Result {
        if self.estado as u8 == BullyState::WaitingOkey as u8 {
            println!(
                "[BULLY-LISTENER-{:?}] TIMEOUT END. Nadie me respondio OK. Soy el nuevo coordinador",
                self.id_nodo
            );
            self.soy_coordinador = true;
            for i in 1..(CANT_MAX_NODOS + 1) {
                let socket: SocketAddr = id_to_addr_write_bully(i)
                    .parse()
                    .expect("Error al formar SocketAddr");
                if let Err(err) = ctx.address().try_send({
                    SenderToUdp {
                        vec: Coordinator::new(self.id_nodo).to_bytes(),
                        socket,
                    }
                }) {
                    println!("[BULLY-LISTENER-{:?}] Error al enviar mensaje de COORDINATOR al nodo {:?} {:?}", self.id_nodo, i, err);
                }
            }

            self.estado = BullyState::NotExecuting;
        } else {
            println!(
                "[BULLY-LISTENER-{:?}] ALGUIEN ME RESPONDIO OK",
                self.id_nodo
            );

            self.estado = BullyState::WaitingCoordinator;
        }
    }
}

/// Mensaje que se recibe de otro nodo cuando su id es mayor y me responde un election
/// El handler setea el estado interno del actor para que sepa que debe esperar a que se
/// anuncie el coordinador
impl Handler<OkeyBully> for BullyListener {
    type Result = ();
    fn handle(&mut self, _msg: OkeyBully, _ctx: &mut Self::Context) -> Self::Result {
        self.estado = BullyState::WaitingCoordinator;
    }
}

/// Mensaje que se recibe de otro nodo cuando comienza el algoritmo de bully
/// El handler envia un Okey al nodo que lo envio
impl Handler<Election> for BullyListener {
    type Result = ();
    fn handle(&mut self, msg: Election, ctx: &mut Self::Context) -> Self::Result {
        let socket: SocketAddr = id_to_addr_write_bully(msg.id_nodo)
            .parse()
            .expect("Error al formar SocketAddr");
        if let Err(err) = ctx.address().try_send({
            SenderToUdp {
                vec: OkeyBully::new(self.id_nodo).to_bytes(),
                socket,
            }
        }) {
            println!(
                "[BULLY-LISTENER-{:?}] Error al enviar OKEY al nodo {:?} | Detalle: {:?}",
                self.id_nodo, msg.id_nodo, err
            );
        }

        if self.conectado {
            if let Err(err) = ctx.address().try_send(StartElection {}) {
                println!(
                    "[BULLY-LISTENER-{:?}] Error al enviar OKEY al nodo {:?} | Detalle: {:?}",
                    self.id_nodo, msg.id_nodo, err
                );
            }
        }
    }
}

/// Mensaje que se recibe cuando un nodo se anuncia nuevo coordinador
/// El handler notifica al actor nodo del id del nuevo coordinador
impl Handler<Coordinator> for BullyListener {
    type Result = ();
    fn handle(&mut self, msg: Coordinator, _ctx: &mut Self::Context) -> Self::Result {
        println!(
            "[BULLY-LISTENER-{:?}] Recibiendo ID {:?} COORDINATOR, se lo NOTIFICO AL NODO",
            msg.id_nodo, self.id_nodo
        );

        self.estado = BullyState::NotExecuting;
        if let Err(err) = self.addr_actor_nodo.try_send(ReceiveNewCoordinator {
            id_nodo_coordinador: msg.id_nodo,
        }) {
            println!("[BULLY-LISTENER-{:?}] Error al enviar mensaje COORDINATOR al nodo {:?} | Detalle: {:?}", self.id_nodo, msg.id_nodo, err);
        }
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct StartElection;

/// Mensaje que se recibe del actor nodo para empezar el algoritmo bully
/// El handler envia un mensaje election a todos los nodos de id superior
impl Handler<StartElection> for BullyListener {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _msg: StartElection, ctx: &mut Self::Context) -> Self::Result {
        println!(
            "[BULLY-LISTENER-{:?}] START ELECTION. EMPIEZO TIMEOUT",
            self.id_nodo
        );

        self.estado = BullyState::WaitingOkey;

        for i in 1..(CANT_MAX_NODOS + 1) {
            if i > self.id_nodo {
                let socket: SocketAddr = id_to_addr_write_bully(i)
                    .parse()
                    .expect("Error al formar SocketAddr");
                if let Err(err) = ctx.address().try_send({
                    SenderToUdp {
                        vec: Election::new(self.id_nodo).to_bytes(),
                        socket,
                    }
                }) {
                    println!("[BULLY-LISTENER-{:?}] Error al enviar ELECTION al ID_NODO {:?} | Detalle: {:?}", self.id_nodo, i, err);
                }
            }
        }

        Box::pin(
            sleep(Duration::from_millis(TIMEOUT_OK_BULLY_MILLIS))
                .into_actor(self)
                .map(move |_result, me, ctx| {
                    if let Err(err) = ctx.address().try_send(TimeoutHandler {}) {
                        println!(
                            "[BULLY-LISTENER-{:?}] Error al crear mi TIMEOUT | Detalle: {:?}",
                            me.id_nodo, err
                        );
                    }
                }),
        )
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct SenderToUdp {
    pub vec: Vec<u8>,
    pub socket: SocketAddr,
}

/// Cuando se envia algo por UDP, el actor se autoenvía este mensaje con el contenido
/// y el socket al que hay que enviarlo
impl Handler<SenderToUdp> for BullyListener {
    type Result = ();

    fn handle(&mut self, msg: SenderToUdp, ctx: &mut Context<Self>) -> Self::Result {
        if !self.conectado {
            return;
        }
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

#[derive(Message, Debug)]
#[rtype(result = "Result<(), ErrorServer>")]
pub struct SetState {
    pub estado: bool,
}

/// Mensaje que se recibe del actor nodo para conectar/desconectar el bully listener
/// En caso de reconectarse, inicia el proceso de broadcast para buscar al nuevo coordinador
impl Handler<SetState> for BullyListener {
    type Result = Result<(), ErrorServer>;

    fn handle(&mut self, msg: SetState, _ctx: &mut Context<Self>) -> Self::Result {
        self.conectado = msg.estado;
        if self.conectado {
            for i in 1..(CANT_MAX_NODOS + 1) {
                let socket: SocketAddr = id_to_addr_write_bully(i).parse().map_err(|_| {
                    ErrorServer::new("Error al formar SocketAddr", TipoError::ErrorGenerico)
                })?;
                _ctx.address()
                    .try_send({
                        SenderToUdp {
                            vec: Ping::new(self.id_nodo).to_bytes(),
                            socket,
                        }
                    })
                    .map_err(|_| {
                        ErrorServer::new("Error al enviar mensaje", TipoError::ErrorGenerico)
                    })?;
            }
        } else {
            self.soy_coordinador = false;
        }

        Ok(())
    }
}

/// Mensaje que se recibe de otro bully listener que esta buscando al coordinador
/// En caso de ser coordinador se envia un PingCord con el id correspondiente
impl Handler<Ping> for BullyListener {
    type Result = ();

    fn handle(&mut self, msg: Ping, ctx: &mut Self::Context) -> Self::Result {
        if self.soy_coordinador {
            let socket: SocketAddr = id_to_addr_write_bully(msg.id_nodo)
                .parse()
                .expect("Error al formar SocketAddr");
            if let Err(err) = ctx.address().try_send({
                SenderToUdp {
                    vec: PingCord::new(self.id_nodo).to_bytes(),
                    socket,
                }
            }) {
                println!(
                    "[BULLY-LISTENER-{:?}] Error al enviar PING_CORD al nodo {:?} | Detalle: {:?}",
                    self.id_nodo, msg.id_nodo, err
                );
            }
        }
    }
}

/// Mensaje que se recibe de otro bully listener que es coordinador
/// El handle notifica al actor nodo del nuevo coordinador
impl Handler<PingCord> for BullyListener {
    type Result = ();

    fn handle(&mut self, msg: PingCord, _ctx: &mut Self::Context) -> Self::Result {
        if let Err(err) = self.addr_actor_nodo.try_send(ReceiveNewCoordinator {
            id_nodo_coordinador: msg.id_nodo,
        }) {
            println!(
                "[BULLY-LISTENER-{:?}] Error al enviar PING_CORD al nodo. | Detalle: {:?}",
                self.id_nodo, err
            );
        }
    }
}
