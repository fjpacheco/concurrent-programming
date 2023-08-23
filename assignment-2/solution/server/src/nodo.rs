use crate::bully_listener::{BullyListener, SetState, StartElection};
use crate::cafetera_listener::{CafeteraListener, ReceiverActorNodo};
use crate::coordinador::{Disconnect, DisconnectToString};
use crate::error_server::{ErrorServer, TipoError};
use crate::mensaje::{
    Abort, Commit, CommitType, Execute, Finish, Mensaje, MensajeBytes, OkeyAbortToCoordinator,
    OkeyToCoordinator, PingCord, Starter, Yes,
};
use crate::utils::{id_to_ctrladdr, SALDO_INICIAL};
use actix::{Actor, ActorFutureExt, AsyncContext, Message};
use compartido::mensajes_cafetera::{
    Error, MensajeCafetera, MensajeCafeteraBytes, OkeyToCafetera, Ping, Restar, Sumar,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::WriteHalf;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_stream::wrappers::LinesStream;

use actix::fut::wrap_future;
use actix::{Addr, Context, ContextFutureSpawner, Handler, StreamHandler};

#[derive(PartialEq, Eq)]
pub enum TransactionState {
    Accepted,
    Wait,
    WaitCommit,
    Locked,
    ToSend,
    Commit,
    Abort,
}

/// Estructura que guarda la información necesaria para
/// completar un pedido correctamente
pub struct Transaction {
    /// addr del actor con el que se gestiona esta transaccion
    pub socket: SocketAddr,
    /// cantidad a ser modificada
    pub cantidad: u32,
    /// Estado de la transaccion
    pub state: TransactionState,
    /// id de la cafetera del nodo
    pub id_cafetera: u8,
    /// id de la cuenta de usuario correspondiente a la transaccion
    pub id_cuenta: u32,
}
/// Estructura que almacena un nodo para realizar los pedidos
pub struct Nodo {
    /// socket tcp al coordinador (nodo handler en realidad)
    stream_cordinador: Arc<Mutex<WriteHalf<TcpStream>>>,
    /// hash map de clave id transaccion y valor id_cuenta
    cuentas: HashMap<u32, Cuenta>,
    /// hashmap de clave id transaccion y valor estructura Transaction para el caso de Restas
    transacciones_resta: HashMap<u32, Transaction>,
    /// id del nodo en funcionamiento
    id_nodo: u8,
    /// Contador de la cantidad de ordenes procesadas por el nodo
    id_orden: u32,
    /// addr del actor cafetera listener
    addr_actor_cafetera: Option<Addr<CafeteraListener>>,
    /// addr del actor bully listener
    addr_actor_bully: Option<Addr<BullyListener>>,
    /// estado de la coneccion
    conectado: bool,
    /// hashmap de clave id transaccion y valor estructura Transaction para el caso de Sumas
    transacciones_suma: HashMap<u32, Transaction>,
    /// id del coordinador actual
    id_coordinador: u8,
}

type IdCafetera = u8;
type IdTransaccion = u32;

#[derive(Debug)]
/// Estructura que almacena el estado de una cuenta de usuario
pub struct Cuenta {
    /// indica si actualmente se está utilizando
    blocked: bool,
    /// saldo restante de la cuenta
    saldo: u32,
    /// hash map que a partir de un id_cafetera retorna el id_transaccion
    transacciones: HashMap<IdCafetera, IdTransaccion>,
}
/// Actor nodo, realiza las acciones que van entre el actor Cafetera-Listener y el Nodo-Handler
/// hace de "servidor local"
impl Actor for Nodo {
    type Context = Context<Self>;
}

impl Nodo {
    /// Incrementa el numero de orden y lo retorna
    fn id_nueva_orden(&mut self) -> u32 {
        self.id_orden += 1;
        self.id_orden
    }

    pub async fn start(id_nodo: u8, id_coordinador: u8) -> Result<(), ErrorServer> {
        let mut stream_cordinador = tokio::net::TcpStream::connect(id_to_ctrladdr(id_coordinador))
            .await
            .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorConexion))?;

        stream_cordinador
            .write(&[id_nodo])
            .await
            .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorConexion))?;

        let addr_actor_nodo = Nodo::create(|ctx| {
            let (read, write_half) = split(stream_cordinador);

            Nodo::add_stream(LinesStream::new(BufReader::new(read).lines()), ctx);
            let write = Arc::new(Mutex::new(write_half));

            Nodo {
                stream_cordinador: write,
                cuentas: HashMap::from([(
                    1,
                    Cuenta {
                        blocked: false,
                        saldo: SALDO_INICIAL,
                        transacciones: HashMap::new(),
                    },
                )]),
                transacciones_resta: HashMap::new(),
                id_orden: 0,
                id_nodo,
                addr_actor_cafetera: None,
                conectado: true,
                transacciones_suma: HashMap::new(),
                addr_actor_bully: None,
                id_coordinador,
            }
        });

        let addr_actor_cafetera = CafeteraListener::start(id_nodo, addr_actor_nodo.clone()).await?;
        addr_actor_nodo.do_send(AddAddrActorCafetera {
            addr_actor_cafetera,
        });

        let addr_actor_bully = BullyListener::start(id_nodo, addr_actor_nodo.clone()).await?;
        addr_actor_nodo.do_send(AddAddrActorBully { addr_actor_bully });

        println!(
            "[NODO-{}] Conectado con el ID_COORDINADOR = {}",
            id_nodo, id_coordinador
        );

        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddAddrActorCafetera {
    pub addr_actor_cafetera: Addr<CafeteraListener>,
}
/// Mensaje para agregar el address del actor cafetera
impl Handler<AddAddrActorCafetera> for Nodo {
    type Result = ();

    fn handle(&mut self, msg: AddAddrActorCafetera, _: &mut Context<Self>) -> Self::Result {
        self.addr_actor_cafetera = Some(msg.addr_actor_cafetera);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddAddrActorBully {
    pub addr_actor_bully: Addr<BullyListener>,
}
/// Mensaje para agregar el address del actor Bully
impl Handler<AddAddrActorBully> for Nodo {
    type Result = ();

    fn handle(&mut self, msg: AddAddrActorBully, _ctx: &mut Context<Self>) -> Self::Result {
        self.addr_actor_bully = Some(msg.addr_actor_bully);
    }
}

/// Mensaje que se recibe escuchando al coordinador, handlea según el tipo de mensaje recibido
impl StreamHandler<Result<String, std::io::Error>> for Nodo {
    fn handle(&mut self, read: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        if let Ok(line) = read {
            let aux = line.as_bytes().to_vec();
            let tipo_mensaje = Mensaje::from_bytes(aux[0]);

            match tipo_mensaje {
                Mensaje::PREPARE => {
                    let mut yes = Yes::from_string(line); // 🚀
                    let id = yes.id_cuenta;
                    yes.id_nodo = self.id_nodo;
                    let cuenta = self.cuentas.get_mut(&id);

                    if let Some(cuenta) = cuenta {
                        cuenta.blocked = true;
                    } else {
                        self.cuentas.insert(
                            id,
                            Cuenta {
                                blocked: true,
                                saldo: SALDO_INICIAL,
                                transacciones: HashMap::new(),
                            },
                        );
                    }

                    if let Err(err) = ctx.address().try_send(SendHandlerToCoordinator {
                        vec: yes.to_string(),
                    }) {
                        println!("ERROR ENVIANDO MENSAJE AL COORDINADOR: {:?}", err);
                    }
                }
                Mensaje::EXECUTE => {
                    let execute = Execute::from_string(line);
                    let socket = self
                        .transacciones_resta
                        .get(&execute.id_transaccion)
                        .expect("Ya se habia insertado la transacción")
                        .socket;
                    println!(
                        "[NODO-{}] Voy a ejecutar EXECUTE en SOCKET {:?}",
                        self.id_nodo, socket
                    );
                    if self
                        .transacciones_resta
                        .get(&execute.get_id_transaccion())
                        .expect("Ya se habia insertado la transacción")
                        .cantidad
                        > self
                            .cuentas
                            .get(&execute.get_id_cuenta())
                            .expect("La cuenta fue insertad previamente")
                            .saldo
                    {
                        self.addr_actor_cafetera
                            .as_ref()
                            .expect("Ya se habia insertado la transacción")
                            .do_send(ReceiverActorNodo {
                                vec: Error::new(0, 0, 0).to_bytes(),
                                socket,
                            });
                        ctx.address().do_send(SendHandlerToCoordinator {
                            vec: Abort::new(
                                self.id_nodo,
                                execute.get_id_cuenta(),
                                execute.id_transaccion,
                                execute.get_id_cafetera(),
                            )
                            .to_string(),
                        });
                        if let Some(transaccion) =
                            self.transacciones_resta.get_mut(&execute.id_transaccion)
                        {
                            transaccion.state = TransactionState::Abort;
                        }
                    } else {
                        if let Some(transaccion) =
                            self.transacciones_resta.get_mut(&execute.id_transaccion)
                        {
                            transaccion.state = TransactionState::Locked;
                        }
                        self.addr_actor_cafetera
                            .as_ref()
                            .expect("Siempre se cuenta con el address del actor cafetera")
                            .do_send(ReceiverActorNodo {
                                vec: OkeyToCafetera::new(0, 0, 0).to_bytes(),
                                socket,
                            })
                    }
                }
                Mensaje::COMMIT => {
                    let commit = Commit::from_string(line);
                    let id = commit.id_cuenta;
                    self.cuentas.entry(id).or_insert(Cuenta {
                        blocked: false,
                        saldo: SALDO_INICIAL,
                        transacciones: HashMap::new(),
                    });
                    let cuenta = self
                        .cuentas
                        .get_mut(&id)
                        .expect("La cuenta ya habia sido insertada");

                    cuenta.blocked = false;
                    if commit.tipo as u8 == CommitType::SUMA as u8 {
                        cuenta.saldo += commit.cantidad;
                        if let Some(transaccion) =
                            self.transacciones_suma.get_mut(&commit.id_transaccion)
                        {
                            transaccion.state = TransactionState::Accepted;
                        }
                    } else {
                        if let Some(transaccion) =
                            self.transacciones_resta.get_mut(&commit.id_transaccion)
                        {
                            transaccion.state = TransactionState::Accepted;
                            // si la transaccion es tuya
                            if let Err(err) = self
                                .addr_actor_cafetera
                                .as_ref()
                                .expect("Siempre se cuenta con el address del actor cafetera")
                                .try_send(ReceiverActorNodo {
                                    vec: OkeyToCafetera::new(0, 0, 0).to_bytes(),
                                    socket: transaccion.socket,
                                })
                            {
                                println!("[NODO-{}] ERROR ENVIANDO MENSAJE AL ACTOR CAFETERA | Detalle: {:?}", self.id_nodo, err);
                            }
                        }
                        cuenta.saldo -= commit.cantidad
                    };

                    ctx.address().do_send(SendHandlerToCoordinator {
                        vec: OkeyToCoordinator::new(
                            self.id_nodo,
                            commit.id_cuenta,
                            commit.id_transaccion,
                            commit.id_cafetera,
                        )
                        .to_string(),
                    });

                    println!(
                        "[NODO-{}] LLEGO COMMIT, CUENTAS: {:?}",
                        self.id_nodo, self.cuentas.iter().map(|(k, v)| (k, v.saldo)).collect::<Vec<_>>()
                    );
                }
                Mensaje::ABORT => {
                    let abort = Abort::from_string(line);
                    let id = abort.id_cuenta;
                    let cuenta = self
                        .cuentas
                        .get_mut(&id)
                        .expect("La cuenta ya fue insertada");
                    cuenta.blocked = false;

                    ctx.address().do_send(SendHandlerToCoordinator {
                        vec: OkeyAbortToCoordinator::new(
                            self.id_nodo,
                            abort.id_cuenta,
                            abort.id_transaccion,
                            abort.get_id_cafetera(),
                        )
                        .to_string(),
                    });
                }

                _ => (),
            }
        } else {
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
        if self.conectado && self.id_nodo != self.id_coordinador {
            if let Err(err) = self
                .addr_actor_bully
                .as_ref()
                .expect("Siempre se cuenta con el address del actor bully")
                .try_send(StartElection {})
            {
                println!(
                    "[NODO-{}] ERROR ENVIANDO MENSAJE AL ACTOR BULLY | Detalle: {:?}",
                    self.id_nodo, err
                );
            }
        }
        self.transacciones_resta
            .iter_mut()
            .for_each(|(_, transaccion)| {
                match transaccion.state {
                    TransactionState::WaitCommit
                    | TransactionState::Wait
                    | TransactionState::Locked => {
                        transaccion.state = TransactionState::Abort;
                        // Error a la cafetera
                        self.addr_actor_cafetera
                            .as_ref()
                            .expect("Siempre se cuenta con el address del actor cafetera")
                            .do_send(ReceiverActorNodo {
                                vec: Error::new(0, 0, 0).to_bytes(),
                                socket: transaccion.socket,
                            });
                    }
                    _ => {}
                }
            });

        self.conectado = false;
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReceiveNewCoordinator {
    ///id del nuevo coordinador
    pub id_nodo_coordinador: u8,
}
/// Hubo un cambio de coordinador, cambia el id y se conecta
impl Handler<ReceiveNewCoordinator> for Nodo {
    type Result = ();

    fn handle(&mut self, msg: ReceiveNewCoordinator, ctx: &mut Context<Self>) -> Self::Result {
        self.id_coordinador = msg.id_nodo_coordinador;
        println!(
            "[NODO-{}] Conectado con el nuevo ID_COORDINADOR = {}",
            self.id_nodo, self.id_coordinador
        );

        let id_coordinador = self.id_coordinador;
        let my_id = self.id_nodo;

        wrap_future::<_, Self>(async move {
            let stream_cordinador = tokio::net::TcpStream::connect(id_to_ctrladdr(id_coordinador))
                .await
                .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorConexion));

            if let Ok(mut stream) = stream_cordinador {
                stream
                    .write(&[my_id])
                    .await
                    .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorConexion))?;
                Ok(stream)
            } else {
                Err(ErrorServer::new(
                    "[ERROR-IMPOSIBLE-OCURRIR] no pude conectarme al nuevo coordinador!!",
                    TipoError::ErrorConexion,
                ))
            }
        })
        .map(|stream_cordinador, this, ctx| {
            if let Ok(stream) = stream_cordinador {
                let (read, write_half) = split(stream);

                ctx.add_stream(LinesStream::new(BufReader::new(read).lines()));
                this.stream_cordinador = Arc::new(Mutex::new(write_half));
                println!(
                    "[NODO-{:?}] Ya me conecté al nuevo cordinador con ID {:?}",
                    this.id_nodo, this.id_coordinador
                );

                this.conectado = true;
            } else {
                println!("[ERROR-IMPOSIBLE-OCURRIR] No me pude conectar al nuevo coordinador!!");
            }
        })
        .wait(ctx);

        for (id_transaccion, transaccion) in self.transacciones_suma.iter_mut() {
            if transaccion.state == TransactionState::ToSend {
                transaccion.state = TransactionState::WaitCommit;
                let finish = Finish::new(
                    self.id_nodo,
                    transaccion.id_cuenta,
                    *id_transaccion,
                    CommitType::SUMA,
                    transaccion.cantidad,
                    transaccion.id_cafetera,
                );
                let _res = ctx.address().try_send(SendHandlerToCoordinator {
                    vec: finish.f_to_string(),
                });
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct SendHandlerToCoordinator {
    vec: String,
}

/// Mensaje cuando hay que enviarle algo al coordinador
impl Handler<SendHandlerToCoordinator> for Nodo {
    type Result = ();

    fn handle(
        &mut self,
        mut msg: SendHandlerToCoordinator,
        ctx: &mut Context<Self>,
    ) -> Self::Result {
        let stream_coor_clone = self.stream_cordinador.clone();
        let addr_actor_bully = self.addr_actor_bully.clone();
        let id_nodo = self.id_nodo;
        msg.vec.push('\n');
        wrap_future::<_, Self>(async move {
            stream_coor_clone
                .lock()
                .await
                .write_all(msg.vec.as_bytes())
                .await
                .map_err(|x| {
                    if x.kind() == std::io::ErrorKind::BrokenPipe {
                        println!(
                            "[NODO-{}] El coordinador se cayó, mando StartElection",
                            id_nodo
                        );
                        if let Err(err) = addr_actor_bully
                            .as_ref()
                            .expect("Siempre se cuenta con el address del actor bully")
                            .try_send(StartElection {})
                        {
                            println!(
                                "[NODO-{}] Error al enviar mensaje al bully | Detalle: {}",
                                id_nodo, err
                            );
                        }
                    }
                    println!("Error al enviar mensaje al coordinador: {}", x);
                })
                .expect("Error al enviar mensaje al coordinador");
        })
        .spawn(ctx);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ReceiveFromCafetera {
    pub msg: Vec<u8>,
    pub socket: SocketAddr,
}

/// Mensaje cuando se recibe algo de cafetera listener, handlea según el tipo del mensaje
impl Handler<ReceiveFromCafetera> for Nodo {
    type Result = ();

    fn handle(&mut self, msg: ReceiveFromCafetera, ctx: &mut Context<Self>) -> Self::Result {
        let tipo_mensaje = MensajeCafetera::from_bytes(msg.msg[0]);
        match tipo_mensaje {
            MensajeCafetera::SUMAR => {
                let mensaje = Sumar::from_bytes(msg.msg.clone());
                let new_id_transaccion: u32 = (self.id_nodo.to_string()
                    + mensaje.id_cafetera.to_string().as_str()
                    + self.id_nueva_orden().to_string().as_str())
                .parse()
                .expect("Error al parsear el id de la transaccion");

                let id = mensaje.get_id_cuenta();
                let cuenta = self.cuentas.get_mut(&id);
                if let Some(cuenta) = cuenta {
                    cuenta
                        .transacciones
                        .insert(mensaje.id_cafetera, new_id_transaccion);
                } else {
                    self.cuentas.insert(
                        id,
                        Cuenta {
                            blocked: false,
                            saldo: SALDO_INICIAL,
                            transacciones: HashMap::from([(
                                mensaje.id_cafetera,
                                new_id_transaccion,
                            )]),
                        },
                    );
                }

                self.transacciones_suma.insert(
                    new_id_transaccion,
                    Transaction {
                        socket: msg.socket,
                        cantidad: mensaje.cantidad_modificar,
                        state: TransactionState::Wait,
                        id_cuenta: id,
                        id_cafetera: mensaje.id_cafetera,
                    },
                );

                if let Err(err) = self
                    .addr_actor_cafetera
                    .as_ref()
                    .expect("Siempre se cuenta con el address del actor cafetera")
                    .try_send(ReceiverActorNodo {
                        vec: OkeyToCafetera::new(0, 0, 0).to_bytes(),
                        socket: msg.socket,
                    })
                {
                    println!(
                        "[NODO-{}] Error al enviar mensaje al actor cafetera | Detalle: {}",
                        self.id_nodo, err
                    );
                }
            }
            MensajeCafetera::RESTAR => {
                if self.conectado {
                    let mensaje = Restar::from_bytes(msg.msg.clone());

                    let new_id_transaccion: u32 = (self.id_nodo.to_string()
                        + mensaje.id_cafetera.to_string().as_str()
                        + self.id_nueva_orden().to_string().as_str())
                    .parse()
                    .expect("Error al parsear el nuevo id de la transaccion");

                    self.transacciones_resta.insert(
                        new_id_transaccion,
                        Transaction {
                            socket: msg.socket,
                            cantidad: mensaje.cantidad_modificar,
                            state: TransactionState::Wait,
                            id_cafetera: mensaje.id_cafetera,
                            id_cuenta: mensaje.id_cuenta,
                        },
                    );

                    let starter = Starter {
                        tipo_mensaje: Mensaje::STARTER.to_bytes(),
                        id_cuenta: mensaje.get_id_cuenta(),
                        id_nodo: self.id_nodo,
                        id_transaccion: new_id_transaccion,
                        id_cafetera: mensaje.get_id_cafetera(),
                    };

                    let id = mensaje.get_id_cuenta();
                    let cuenta = self.cuentas.get_mut(&id);
                    if let Some(cuenta) = cuenta {
                        cuenta
                            .transacciones
                            .insert(mensaje.id_cafetera, new_id_transaccion);
                    } else {
                        self.cuentas.insert(
                            id,
                            Cuenta {
                                blocked: false,
                                saldo: SALDO_INICIAL,
                                transacciones: HashMap::from([(
                                    mensaje.id_cafetera,
                                    new_id_transaccion,
                                )]),
                            },
                        );
                    }

                    let _res = ctx.address().try_send(SendHandlerToCoordinator {
                        vec: starter.to_string(),
                    });
                } else if let Err(err) = self
                    .addr_actor_cafetera
                    .as_ref()
                    .expect("Siempre se cuenta con el address del actor cafetera")
                    .try_send(ReceiverActorNodo {
                        vec: Error::new(0, 0, 0).to_bytes(),
                        socket: msg.socket,
                    })
                {
                    println!(
                        "[NODO-{}] Error al enviar mensaje al actor cafetera | Detalle: {}",
                        self.id_nodo, err
                    );
                }
            }
            MensajeCafetera::PING => {
                //enviar al coordinador otro ping, para ver si seguimos conectados
                let mensaje = Ping::from_bytes(msg.msg);

                let ping = PingCord {
                    tipo_mensaje: Mensaje::PING.to_bytes(),
                    id_cuenta: mensaje.get_id_cuenta(),
                    id_nodo: self.id_nodo,
                    id_transaccion: 0,
                    id_cafetera: mensaje.get_id_cafetera(),
                };
                // SI FALLA ENTRAR EN MODO DESCONECTADO
                let _res = ctx.address().try_send(SendHandlerToCoordinator {
                    vec: ping.to_string(),
                });
            }

            MensajeCafetera::OKEY => {
                let mensaje = OkeyToCafetera::from_bytes(msg.msg);
                let id_cuenta = mensaje.get_id_cuenta();

                let id_transaccion = self
                    .cuentas
                    .get(&id_cuenta)
                    .expect("La cuenta se ha borrado")
                    .transacciones
                    .get(&mensaje.id_cafetera)
                    .expect("La cuenta se ha borrado");
                if let Some(transaccion_suma) = self.transacciones_suma.get_mut(id_transaccion) {
                    if self.conectado {
                        transaccion_suma.state = TransactionState::WaitCommit;
                        let finish = Finish::new(
                            self.id_nodo,
                            id_cuenta,
                            *id_transaccion,
                            CommitType::SUMA,
                            transaccion_suma.cantidad,
                            mensaje.id_cafetera,
                        );
                        let _res = ctx.address().try_send(SendHandlerToCoordinator {
                            vec: finish.f_to_string(),
                        });
                    } else {
                        transaccion_suma.state = TransactionState::ToSend;
                    }
                } else if let Some(transaccion_resta) =
                    self.transacciones_resta.get_mut(id_transaccion)
                {
                    if self.conectado {
                        transaccion_resta.state = TransactionState::WaitCommit;
                        let finish = Finish::new(
                            self.id_nodo,
                            id_cuenta,
                            *id_transaccion,
                            CommitType::RESTA,
                            transaccion_resta.cantidad,
                            mensaje.id_cafetera,
                        );
                        let _res = ctx.address().try_send(SendHandlerToCoordinator {
                            vec: finish.f_to_string(),
                        });
                    } else {
                        transaccion_resta.state = TransactionState::Abort;
                        self.addr_actor_cafetera
                            .as_ref()
                            .expect("Error al obtener la direccion del actor cafetera")
                            .do_send(ReceiverActorNodo {
                                vec: Error::new(0, 0, 0).to_bytes(),
                                socket: transaccion_resta.socket,
                            });
                    }
                } else {
                    // matate!
                }
            }
            MensajeCafetera::ERROR => {
                let mensaje = Error::from_bytes(msg.msg);
                let id_cuenta = mensaje.get_id_cuenta();
                let id_transaccion = self
                    .cuentas
                    .get(&id_cuenta)
                    .expect("Cuenta previamente creada")
                    .transacciones
                    .get(&mensaje.id_cafetera)
                    .expect("Transaccion previamente creada");

                if let Some(transaccion_suma) = self.transacciones_suma.get_mut(id_transaccion) {
                    transaccion_suma.state = TransactionState::Abort;
                } else if let Some(transaccion_resta) =
                    self.transacciones_resta.get_mut(id_transaccion)
                {
                    if self.conectado {
                        println!(
                            "[NODO-{}] MANDO UN abort AL COORDINADOR sobre ID_TRANSACCION: {}",
                            self.id_nodo, id_transaccion
                        );
                        transaccion_resta.state = TransactionState::Abort;
                        let finish = Abort::new(
                            self.id_nodo,
                            id_cuenta,
                            *id_transaccion,
                            mensaje.id_cafetera,
                        );
                        if let Err(err) = ctx.address().try_send(SendHandlerToCoordinator {
                            vec: finish.to_string(),
                        }) {
                            println!(
                                "[NODO-{}] Error al enviar mensaje al coordinador | Detalle: {:?}",
                                self.id_nodo, err
                            );
                        }
                        transaccion_resta.state = TransactionState::Abort;
                    }
                }
            }
            MensajeCafetera::DESCONOCIDO => {}
            MensajeCafetera::DESCONECTAR => {
                //chequeamos si somos el cordiandor
                if self.id_nodo == self.id_coordinador {
                    //soy el cordinador, desconecto los nodo-handlers
                    let mensaje = Disconnect {
                        tipo_mensaje: Mensaje::DISCONNECT as u8,
                    };
                    if let Err(err) = ctx.address().try_send(SendHandlerToCoordinator {
                        vec: mensaje.to_string(),
                    }) {
                        println!(
                            "[NODO-{}] Error al enviar mensaje al coordinador | Detalle: {:?}",
                            self.id_nodo, err
                        );
                    }

                    if let Err(err) = self
                        .addr_actor_bully
                        .as_ref()
                        .expect("Siempre se dispone de un address bully")
                        .try_send(SetState { estado: false })
                    {
                        println!(
                            "[NODO-{}] Error al enviar mensaje al bully | Detalle: {:?}",
                            self.id_nodo, err
                        );
                    }
                } else {
                    self.conectado = false;
                    let arc = self.stream_cordinador.clone();
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
            MensajeCafetera::CONECTAR => {
                if let Err(err) = self
                    .addr_actor_bully
                    .as_ref()
                    .expect("Siempre se dispone de un bully")
                    .try_send(SetState { estado: true })
                {
                    println!(
                        "[NODO-{}] Error al enviar mensaje al bully | Detalle: {:?}",
                        self.id_nodo, err
                    );
                }
                self.conectado = true;
                let _arc = self.stream_cordinador.clone();
            }
        }
    }
}
