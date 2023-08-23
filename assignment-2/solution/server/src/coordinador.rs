use std::collections::HashMap;
use std::sync::Arc;
use std::vec;

use actix::{Actor, Context, Handler, Message, StreamHandler};
use actix::{Addr, AsyncContext};

use tokio::io::{split, AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_stream::wrappers::LinesStream;

use crate::error_server::{ErrorServer, TipoError};
use crate::mensaje::{
    Abort, Commit, CommitType, Execute, Finish, MensajeBytes, OkeyAbortToCoordinator,
    OkeyToCoordinator, PingCord, Prepare, Starter, Yes,
};
use crate::nodo_handler::{NodoHandler, ReceiverFromCoordinador, Shutdown};
use crate::utils::id_to_ctrladdr;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum TransactionState {
    Uninitialized,
    Wait,
    Execute,
    Commit,
    Abort,
    Done,
}
/// Información de importancia para el coordinador sobre una transacción
struct TransactionCoordinator {
    /// Estado en el que se encuentra la transaccion
    status: TransactionState,
    /// vector de id_nodos que respondieron al prepare
    yes_nodos: Vec<u8>,
    /// vector de id_nodos que respondieron al commit
    ok_nodos: Vec<u8>,
    /// id_nodo que inicio la transaccion
    from_id_nodo: u8,
    /// cuenta a la que corresponde la transaccion
    id_cuenta: u32,
    /// tipo de orden a realizar
    tipo: CommitType,
    /// id de la cafetera del nodo
    id_cafetera: u8,
}
/// Estructura que guarda la información general del servidor usada por el coordiandor.
pub struct Coordinador {
    /// Hash con clave id_nodo y valor el address del actor nodo-handler
    addr_nodos: HashMap<u8, Addr<NodoHandler>>,
    /// Hash con clave id_transaccion y como valor la estructura TransactionCoordinator
    transacciones: HashMap<u32, TransactionCoordinator>,
    /// Hash de clave id_cuenta y valor vector de id_transacciones
    /// indica que transacciones estan pendientes por loquearse ante el uso de una misma cuenta
    cola: HashMap<u32, Vec<u32>>,
    /// Estado de la conección
    conectado: bool,
}

impl Actor for Coordinador {
    type Context = Context<Self>;
}

impl Coordinador {
    ///Inicializar el socket TCP
    pub async fn create_listener(id: u8) -> Result<TcpListener, ErrorServer> {
        TcpListener::bind(id_to_ctrladdr(id))
            .await
            .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorConexion))
    }
    /// Crea el actor Coordinador y por cada conección entrante al socket tcp se crea un actor nodo-handler
    pub async fn start_listener(listener: TcpListener) -> Result<(), ErrorServer> {
        let coordinador_addr = Coordinador {
            addr_nodos: HashMap::new(),
            transacciones: HashMap::new(),
            cola: HashMap::new(),
            conectado: true,
        }
        .start();

        while let Ok((mut stream, addr)) = listener.accept().await {
            let coordinador_addr_clone = coordinador_addr.clone();
            let id_nodo: u8 = stream
                .read_u8()
                .await
                .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorGenerico))?;

            println!(
                "[COORDINADOR] Conexion establecidada con ID_NODO = {:?}",
                id_nodo
            );

            coordinador_addr_clone
                .try_send(SetState { estado: true })
                .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorGenerico))?;

            let nodo_addr = NodoHandler::create(|ctx| {
                let (read, write_half) = split(stream);

                NodoHandler::add_stream(LinesStream::new(BufReader::new(read).lines()), ctx);
                let write = Arc::new(Mutex::new(write_half));
                NodoHandler {
                    addr,
                    write,
                    addr_coordinador: coordinador_addr_clone,
                    id_nodo,
                    conectado: true,
                }
            });

            coordinador_addr
                .send(AddNodo { nodo_addr, id_nodo })
                .await
                .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorGenerico))?;
        }
        Ok(())
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
/// Se agrega en el vector de addr de nodo-handlers uno nuevo
pub struct AddNodo {
    nodo_addr: Addr<NodoHandler>,
    id_nodo: u8,
}

impl Handler<AddNodo> for Coordinador {
    type Result = ();
    fn handle(&mut self, msg: AddNodo, _ctx: &mut Self::Context) -> Self::Result {
        self.addr_nodos.insert(msg.id_nodo, msg.nodo_addr);
    }
}
/// Recibe Starter solo para las operaciones de RESTA!
impl Handler<Starter> for Coordinador {
    type Result = ();

    fn handle(&mut self, msg: Starter, _ctx: &mut Context<Self>) -> Self::Result {
        if !self.conectado {
            return;
        }

        println!(
            "[COORDINADOR] Recibí STARTER de ID_NODO = {:?}",
            msg.id_nodo
        );

        let id_transaccion = msg.id_transaccion;
        let id_cuenta = msg.id_cuenta;

        self.transacciones.insert(
            id_transaccion,
            TransactionCoordinator {
                status: TransactionState::Uninitialized,
                yes_nodos: vec![],
                ok_nodos: vec![],
                from_id_nodo: msg.id_nodo,
                id_cuenta: msg.id_cuenta,
                tipo: CommitType::RESTA,
                id_cafetera: msg.id_cafetera,
            },
        );

        if let Some(cuenta) = self.cola.get_mut(&id_cuenta) {
            cuenta.push(id_transaccion);
            if cuenta.len() > 1 {
                return;
            }
        } else {
            self.cola.insert(id_cuenta, vec![id_transaccion]);
        }

        self.transacciones
            .get_mut(&id_transaccion)
            .expect("Se había insertado la transacción anteriormente")
            .status = TransactionState::Wait;

        for (_, addr) in self.addr_nodos.iter() {
            let prepare = Prepare::from_start(msg.clone());
            let res = addr
                .try_send(ReceiverFromCoordinador {
                    string: prepare.to_string(),
                })
                .map_err(|x| ErrorServer::new(&x.to_string(), TipoError::ErrorGenerico));
            if let Err(res) = res {
                println!("[HANDLER-STARTER] error: {:?}", res);
            }
        }
    }
}
/// Recibe un Yes, se agrega en la correspondiente transaccion
impl Handler<Yes> for Coordinador {
    type Result = ();

    fn handle(&mut self, msg: Yes, _ctx: &mut Context<Self>) -> Self::Result {
        if !self.conectado {
            return;
        }
        println!("[COORDINADOR] Recibí YES de ID_NODO = {:?}", msg.id_nodo);
        if let Some(transaccion) = self.transacciones.get_mut(&msg.id_transaccion) {
            transaccion.yes_nodos.push(msg.id_nodo);

            if transaccion.yes_nodos.len() == self.addr_nodos.len() {
                transaccion.status = TransactionState::Execute;
                if let Err(err) = self
                    .addr_nodos
                    .get(&transaccion.from_id_nodo)
                    .expect("Siempre se obtendra la address")
                    .try_send(ReceiverFromCoordinador {
                        string: Execute::new(
                            transaccion.from_id_nodo,
                            msg.id_cuenta,
                            msg.id_transaccion,
                            msg.id_cafetera,
                        )
                        .to_string(),
                    })
                {
                    println!(
                        "[COORDINADOR] Error al enviar EXECUTE al ID_NODO = {:?} | Detalle: {:?}",
                        transaccion.from_id_nodo, err
                    );
                }
            }
        }
    }
}
/// Recibi un ping, no hago nada ya que el nodo por ser tcp sabe que esta todo ok
impl Handler<PingCord> for Coordinador {
    type Result = ();

    fn handle(&mut self, msg: PingCord, _ctx: &mut Context<Self>) -> Self::Result {
        if !self.conectado {
            return;
        }

        println!("[COORDINADOR] Recibí PING de ID_NODO = {:?}", msg.id_nodo);
    }
}
/// Recibo un finish, se actualiza la transaccion y se envia commit a los nodos-handlers
impl Handler<Finish> for Coordinador {
    type Result = ();

    fn handle(&mut self, msg: Finish, _ctx: &mut Context<Self>) -> Self::Result {
        if !self.conectado {
            return;
        }

        println!("[COORDINADOR] Recibí FINISH de ID_NODO = {:?}", msg.id_nodo);

        if msg.tipo as u8 == CommitType::SUMA as u8 {
            self.transacciones.insert(
                msg.id_transaccion,
                TransactionCoordinator {
                    status: TransactionState::Wait,
                    yes_nodos: vec![],
                    ok_nodos: vec![],
                    from_id_nodo: msg.id_nodo,
                    id_cuenta: msg.id_cuenta,
                    tipo: CommitType::SUMA,
                    id_cafetera: msg.id_cafetera,
                },
            );
        }
        if let Some(x) = self.transacciones.get_mut(&msg.id_transaccion) {
            x.status = TransactionState::Commit
        };

        for (_, addr) in self.addr_nodos.iter() {
            let commit = Commit::new(
                msg.id_nodo,
                msg.id_cuenta,
                msg.id_transaccion,
                msg.tipo,
                msg.cantidad,
                msg.id_cafetera,
            );
            if let Err(err) = addr.try_send(ReceiverFromCoordinador {
                string: commit.set_to_string(),
            }) {
                println!(
                    "[COORDINADOR] Error al enviar COMMIT al ID_NODO = {:?} | Detalle: {:?}",
                    msg.id_nodo, err
                );
            }
        }
    }
}
/// Recibo un okey, actualizo el vector de Ok para la transaccion correspondiente,
/// valida si ya se tienen todos los ok
impl Handler<OkeyToCoordinator> for Coordinador {
    type Result = ();

    fn handle(&mut self, msg: OkeyToCoordinator, _ctx: &mut Context<Self>) -> Self::Result {
        if !self.conectado {
            return;
        }

        println!("[COORDINADOR] Recibí OK de ID_NODO = {:?}", msg.id_nodo);
        if let Some(x) = self.transacciones.get_mut(&msg.id_transaccion) {
            x.ok_nodos.push(msg.id_nodo);
        }

        if let Some(x) = self.transacciones.get_mut(&msg.id_transaccion) {
            if x.ok_nodos.len() == self.addr_nodos.len() {
                x.status = TransactionState::Done;
                x.ok_nodos = vec![];
                if x.tipo as u8 == CommitType::SUMA as u8 {
                    return;
                }
                if let Some(pendientes_cuenta) = self.cola.get_mut(&x.id_cuenta) {
                    pendientes_cuenta.remove(0);
                    if !pendientes_cuenta.is_empty() {
                        let id_transaccion_from_queue = pendientes_cuenta[0];
                        if let Some(transaccion) =
                            self.transacciones.get_mut(&id_transaccion_from_queue)
                        {
                            transaccion.status = TransactionState::Wait;
                            for (_, addr) in self.addr_nodos.iter() {
                                let prepare = Prepare::new(
                                    transaccion.from_id_nodo,
                                    transaccion.id_cuenta,
                                    id_transaccion_from_queue,
                                    msg.id_cafetera,
                                );
                                if let Err(err) = addr
                                    .try_send(ReceiverFromCoordinador {
                                        string: prepare.to_string(),
                                    })
                                    .map_err(|x| {
                                        ErrorServer::new(&x.to_string(), TipoError::ErrorGenerico)
                                    })
                                {
                                    println!(
                                        "[COORDINADOR] Error al enviar PREPARE al ID_NODO = {:?} | Detalle: {:?}",
                                        transaccion.from_id_nodo, err
                                    );
                                }
                            }
                        } else {
                            println!(
                                "[COORDINADOR] No existe la transaccion con ID_TRANSACCION = {:?}",
                                id_transaccion_from_queue
                            );
                        }
                    }
                } else {
                    println!(
                        "[COORDINADOR] No existe la cola de pendientes para ID_CUENTA = {:?}",
                        x.id_cuenta
                    );
                }
            }
        } else {
            println!("[COORDINADOR] Recibi OK de ID_NODO = {:?} pero no existe la transaccion con ID_TRANSACCION = {:?}", msg.id_nodo, msg.id_transaccion);
        }
    }
}
/// Recibo un okey, actualizo el vector de OkAbort para la transaccion correspondiente,
/// valida si ya se tienen todos los OkAbort
impl Handler<OkeyAbortToCoordinator> for Coordinador {
    type Result = ();

    fn handle(&mut self, msg: OkeyAbortToCoordinator, _ctx: &mut Context<Self>) -> Self::Result {
        if !self.conectado {
            return;
        }

        println!(
            "[COORDINADOR] Recibí OK_ABORT de ID_NODO = {:?}",
            msg.id_nodo
        );
        if let Some(x) = self.transacciones.get_mut(&msg.id_transaccion) {
            x.ok_nodos.push(msg.id_nodo);
        }

        let mut transaccion = match self.transacciones.get_mut(&msg.id_transaccion) {
            Some(x) => x,
            None => {
                println!("[COORDINADOR] Recibi OK_ABORT de ID_NODO = {:?} pero no existe la transaccion con ID_TRANSACCION = {:?}", msg.id_nodo, msg.id_transaccion);
                return;
            }
        };

        if transaccion.ok_nodos.len() == self.addr_nodos.len() {
            transaccion.status = TransactionState::Abort;
            transaccion.ok_nodos = vec![];
            let pendientes_cuenta = match self.cola.get_mut(&transaccion.id_cuenta) {
                Some(x) => x,
                None => {
                    println!(
                        "[COORDINADOR] No existe la cola de pendientes para ID_CUENTA = {:?}",
                        transaccion.id_cuenta
                    );
                    return;
                }
            };
            pendientes_cuenta.remove(0);
            if !pendientes_cuenta.is_empty() {
                let id_transaccion = pendientes_cuenta[0];
                let transaccion = match self.transacciones.get_mut(&id_transaccion) {
                    Some(x) => x,
                    None => {
                        println!(
                            "[COORDINADOR] No existe la transaccion con ID_TRANSACCION = {:?}",
                            id_transaccion
                        );
                        return;
                    }
                };

                transaccion.status = TransactionState::Wait;
                for (_, addr) in self.addr_nodos.iter() {
                    let prepare = Prepare::new(
                        transaccion.from_id_nodo,
                        transaccion.id_cuenta,
                        id_transaccion,
                        msg.id_cafetera,
                    );
                    if let Err(err) = addr.try_send(ReceiverFromCoordinador {
                        string: prepare.to_string(),
                    }) {
                        println!(
                                "[COORDINADOR] Error al enviar PREPARE al ID_NODO = {:?} | Detalle: {:?}",
                                transaccion.from_id_nodo, err
                            );
                    }
                }
            }
        }
    }
}
/// Handler de Abort, se actualiza la transaccion y se notifica a todos los nodos-handlers
impl Handler<Abort> for Coordinador {
    type Result = ();

    fn handle(&mut self, msg: Abort, _ctx: &mut Context<Self>) -> Self::Result {
        if !self.conectado {
            return;
        }

        println!("[COORDINADOR] Recibí ABORT de ID_NODO = {:?}", msg.id_nodo);
        if let Some(x) = self.transacciones.get_mut(&msg.id_transaccion) {
            x.status = TransactionState::Abort;
        }

        self.addr_nodos.iter().for_each(|(_, addr)| {
            if let Err(err) = addr.try_send(ReceiverFromCoordinador {
                string: msg.to_string(),
            }) {
                println!(
                    "[COORDINADOR] Error al enviar ABORT al ID_NODO = {:?} | Detalle: {:?}",
                    msg.id_nodo, err
                );
            }
        });
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Handler de disconnect, hay que simular desconección de red
/// las transacciones pendientes deben abortarse
pub struct DisconnectNodo {
    pub id_nodo: u8,
}

impl Handler<DisconnectNodo> for Coordinador {
    type Result = ();

    fn handle(&mut self, msg: DisconnectNodo, ctx: &mut Self::Context) -> Self::Result {
        if !self.conectado {
            return;
        }

        self.addr_nodos.remove(&msg.id_nodo);
        for (id_transaccion, transaccion) in self.transacciones.iter_mut() {
            if transaccion.from_id_nodo == msg.id_nodo
                && transaccion.status as u8 != TransactionState::Abort as u8
                && transaccion.status as u8 != TransactionState::Done as u8
            {
                transaccion.status = TransactionState::Abort;

                if let Err(err) = ctx.address().try_send(Abort::new(
                    transaccion.from_id_nodo,
                    transaccion.id_cuenta,
                    *id_transaccion,
                    transaccion.id_cafetera,
                )) {
                    println!(
                        "[COORDINADOR] Error al enviar ABORT al ID_NODO = {:?} | Detalle: {:?}",
                        transaccion.from_id_nodo, err
                    );
                }
            }
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub tipo_mensaje: u8,
}
/// Llega un disconnect, hay que limpiar el estado del servidor y apagar
/// todos los actores nodo-handlers
impl Handler<Disconnect> for Coordinador {
    type Result = ();

    fn handle(&mut self, _: Disconnect, _: &mut Self::Context) -> Self::Result {
        for (id_nodo, addr) in self.addr_nodos.iter() {
            if let Err(err) = addr.try_send(Shutdown {}) {
                println!(
                    "[COORDINADOR] Error al enviar SHUTDOWN al ID_NODO = {:?} | Detalle: {:?}",
                    id_nodo, err
                );
            }
        }

        self.conectado = false;
        self.addr_nodos = HashMap::new();
        self.transacciones = HashMap::new();
        self.cola = HashMap::new();
    }
}

pub trait DisconnectToString {
    fn to_string(&self) -> String;
    fn get_tipo_mensaje(&self) -> u8;
}

impl DisconnectToString for Disconnect {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn to_string(&self) -> String {
        self.tipo_mensaje.to_string()
    }
}

// impl Display for Disconnect {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.tipo_mensaje)
//     }
// }

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct SetState {
    pub estado: bool,
}
/// Se recibe un mensaje para cambiar el estado de la conexion
impl Handler<SetState> for Coordinador {
    type Result = ();

    fn handle(&mut self, msg: SetState, _: &mut Self::Context) -> Self::Result {
        self.conectado = msg.estado;
    }
}
