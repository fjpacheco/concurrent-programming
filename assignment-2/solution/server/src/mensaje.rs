use actix::Message;

#[derive(Debug)]
/// Mensajes que serán enviados entre el coordinador y los nodos
pub enum Mensaje {
    STARTER,
    PREPARE,
    YES,
    EXECUTE,
    FINISH,
    COMMIT,
    OKEY,
    ABORT,
    PING,
    OKEYABORT,
    DISCONNECT,
    UNKNOWN,
}

impl Mensaje {
    pub fn from_bytes(byte: u8) -> Mensaje {
        match byte {
            48_u8 => Mensaje::STARTER,
            49_u8 => Mensaje::PREPARE,
            50_u8 => Mensaje::YES,
            51_u8 => Mensaje::EXECUTE,
            52_u8 => Mensaje::FINISH,
            53_u8 => Mensaje::COMMIT,
            54_u8 => Mensaje::OKEY,
            55_u8 => Mensaje::ABORT,
            56_u8 => Mensaje::PING,
            57_u8 => Mensaje::OKEYABORT,
            58_u8 => Mensaje::DISCONNECT,
            _ => Mensaje::UNKNOWN,
        }
    }

    pub fn to_bytes(&self) -> u8 {
        match self {
            Mensaje::STARTER => 0_u8,
            Mensaje::PREPARE => 1_u8,
            Mensaje::YES => 2_u8,
            Mensaje::EXECUTE => 3_u8,
            Mensaje::FINISH => 4_u8,
            Mensaje::COMMIT => 5_u8,
            Mensaje::OKEY => 6_u8,
            Mensaje::ABORT => 7_u8,
            Mensaje::PING => 8_u8,
            Mensaje::OKEYABORT => 9_u8,
            Mensaje::DISCONNECT => 10_u8,
            Mensaje::UNKNOWN => 11_u8,
        }
    }
}

pub trait MensajeBytes {
    fn new(id_nodo: u8, id_cuenta: u32, id_transaccion: u32, id_cafetera: u8) -> Self;
    fn get_tipo_mensaje(&self) -> u8;
    fn get_id_nodo(&self) -> u8;
    fn get_id_cuenta(&self) -> u32;
    fn get_id_transaccion(&self) -> u32;
    fn get_id_cafetera(&self) -> u8;

    fn to_string(&self) -> String {
        let mut result = String::new();
        let tipo_mensaje = self.get_tipo_mensaje().to_string();
        let id_nodo = self.get_id_nodo().to_string();
        let id_cuenta = self.get_id_cuenta().to_string();
        let id_transaccion = self.get_id_transaccion().to_string();
        let id_cafetera = self.get_id_cafetera().to_string();

        result.push_str(&tipo_mensaje);
        result.push('-');
        result.push_str(&id_nodo);
        result.push('-');
        result.push_str(&id_cuenta);
        result.push('-');
        result.push_str(&id_transaccion);
        result.push('-');
        result.push_str(&id_cafetera);

        result
    }

    fn from_string(string: String) -> Self
    where
        Self: Sized,
    {
        // split for "-" in string
        let bytes: Vec<String> = string.split('-').map(|x| x.to_string()).collect();

        let id_nodo: u8 = bytes[1].parse().expect("Error parsing id_nodo");
        let id_cuenta: u32 = bytes[2].parse().expect("Error parsing id_cuenta");
        let id_transaccion: u32 = bytes[3].parse().expect("Error parsing id_transaccion");
        let id_cafetera: u8 = bytes[4].parse().expect("Error parsing id_cafetera");

        Self::new(id_nodo, id_cuenta, id_transaccion, id_cafetera)
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
///Mensaje que envia un nodo al coordinador al comenzar un pedido del tipo Resta
pub struct Starter {
    /// Tipo de mensaje (Starter)
    pub tipo_mensaje: u8,
    /// id del Nodo que inicia el pedido
    pub id_nodo: u8,
    /// id de la cuenta de usuario
    pub id_cuenta: u32,
    /// id de la transaccion iniciada
    pub id_transaccion: u32,
    /// id de la cafetera del nodo
    pub id_cafetera: u8,
}

impl MensajeBytes for Starter {
    fn new(id_nodo: u8, id_cuenta: u32, id_transaccion: u32, id_cafetera: u8) -> Starter {
        Starter {
            tipo_mensaje: Mensaje::STARTER.to_bytes(),
            id_nodo,
            id_cuenta,
            id_transaccion,
            id_cafetera,
        }
    }

    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_id_transaccion(&self) -> u32 {
        self.id_transaccion
    }

    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Mensaje enviado por el coordinador a todos los nodos al momento de recibir un Starter
pub struct Prepare {
    /// tipo de mensaje (prepare)
    pub tipo_mensaje: u8,
    /// id del nodo que inicio el starter
    pub id_nodo: u8,
    /// id de la cuenta de usuario
    pub id_cuenta: u32,
    /// id de la transaccion iniciada
    pub id_transaccion: u32,
    /// id de la cafetera correspondiente al nodo
    pub id_cafetera: u8,
}

impl MensajeBytes for Prepare {
    fn new(id_nodo: u8, id_cuenta: u32, id_transaccion: u32, id_cafetera: u8) -> Prepare {
        Prepare {
            tipo_mensaje: Mensaje::PREPARE.to_bytes(),
            id_nodo,
            id_cuenta,
            id_transaccion,
            id_cafetera,
        }
    }

    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_id_transaccion(&self) -> u32 {
        self.id_transaccion
    }

    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
}

impl Prepare {
    pub fn from_start(starter: Starter) -> Prepare {
        Prepare {
            tipo_mensaje: Mensaje::PREPARE.to_bytes(),
            id_nodo: starter.id_nodo,
            id_cuenta: starter.id_cuenta,
            id_transaccion: starter.id_transaccion,
            id_cafetera: starter.id_cafetera,
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Mensaje enviado por los nodos al coordiandor luego de recibir un prepare
pub struct Yes {
    /// tipo de mensaje (yes)
    pub tipo_mensaje: u8,
    /// id del nodo que comenzo la transaccion
    pub id_nodo: u8,
    /// id de la cuenta de usuario
    pub id_cuenta: u32,
    /// id de la transaccion
    pub id_transaccion: u32,
    /// id de la cafetera correspondiente al nodo
    pub id_cafetera: u8,
}

impl MensajeBytes for Yes {
    fn new(id_nodo: u8, id_cuenta: u32, id_transaccion: u32, id_cafetera: u8) -> Yes {
        Yes {
            tipo_mensaje: Mensaje::YES.to_bytes(),
            id_nodo,
            id_cuenta,
            id_transaccion,
            id_cafetera,
        }
    }

    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_id_transaccion(&self) -> u32 {
        self.id_transaccion
    }

    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Mensaje enviado del coordinador al nodo que envió el starter
pub struct Execute {
    /// tipode mensaje (execute)
    pub tipo_mensaje: u8,
    /// id del nodo qeu envio el starter
    pub id_nodo: u8,
    /// id de la cuenta de usuario
    pub id_cuenta: u32,
    /// id de la transaccion a realizar
    pub id_transaccion: u32,
    /// id de la cafetera del nodo
    pub id_cafetera: u8,
}

impl MensajeBytes for Execute {
    fn new(id_nodo: u8, id_cuenta: u32, id_transaccion: u32, id_cafetera: u8) -> Execute {
        Execute {
            tipo_mensaje: Mensaje::EXECUTE.to_bytes(),
            id_nodo,
            id_cuenta,
            id_transaccion,
            id_cafetera,
        }
    }
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_id_transaccion(&self) -> u32 {
        self.id_transaccion
    }
    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Mensaje enviado por el nodo que realizó el pedido una vez finalizado
pub struct Finish {
    /// tipo de mensaje (finish)
    pub tipo_mensaje: u8,
    /// id del nodo que finalizo el pedido
    pub id_nodo: u8,
    /// id dela cuenta de usuario
    pub id_cuenta: u32,
    /// id de latransaccion finalizada
    pub id_transaccion: u32,
    /// tipo de pedido finalizado
    pub tipo: CommitType,
    /// cantidad de creditos implicados
    pub cantidad: u32,
    /// id de la cafetera del nodo
    pub id_cafetera: u8,
}

impl Finish {
    pub fn new(
        id_nodo: u8,
        id_cuenta: u32,
        id_transaccion: u32,
        tipo: CommitType,
        cantidad: u32,
        id_cafetera: u8,
    ) -> Finish {
        Finish {
            tipo_mensaje: Mensaje::FINISH.to_bytes(),
            id_nodo,
            id_cuenta,
            id_transaccion,
            tipo,
            cantidad,
            id_cafetera,
        }
    }

    pub fn f_to_string(&self) -> String {
        let mut result = String::new();
        let tipo_mensaje = self.get_tipo_mensaje().to_string();
        let id_nodo = self.get_id_nodo().to_string();
        let id_cuenta = self.get_id_cuenta().to_string();
        let id_transaccion = self.get_id_transaccion().to_string();
        let tipo = (self.tipo as u8).to_string();
        let cantidad = self.cantidad.to_string();
        let id_cafetera = self.id_cafetera.to_string();
        result.push_str(&tipo_mensaje);
        result.push('-');
        result.push_str(&id_nodo);
        result.push('-');
        result.push_str(&id_cuenta);
        result.push('-');
        result.push_str(&id_transaccion);
        result.push('-');
        result.push_str(&tipo);
        result.push('-');
        result.push_str(&cantidad);
        result.push('-');
        result.push_str(&id_cafetera);
        result
    }

    pub fn from_string(string: String) -> Finish {
        // split for "-" in string
        let bytes: Vec<String> = string.split('-').map(|x| x.to_string()).collect();

        let id_nodo: u8 = bytes[1].parse().expect("Error al parsear id_nodo");
        let id_cuenta: u32 = bytes[2].parse().expect("Error al parsear id_cuenta");
        let id_transaccion: u32 = bytes[3].parse().expect("Error al parsear id_transaccion");
        let tipo = CommitType::from_bytes(bytes[4].parse().expect("Error al parsear tipo"));
        let cantidad: u32 = bytes[5].parse().expect("Error al parsear cantidad");
        let id_cafetera: u8 = bytes[6].parse().expect("Error al parsear id_cafetera");

        Finish::new(
            id_nodo,
            id_cuenta,
            id_transaccion,
            tipo,
            cantidad,
            id_cafetera,
        )
    }

    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_id_transaccion(&self) -> u32 {
        self.id_transaccion
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Mensaje enviado por los nodos al coordinador luego de recibir un commit
pub struct OkeyToCoordinator {
    /// tipo de mensaje (OkeyToCoordinator)
    pub tipo_mensaje: u8,
    /// id del nodo que realizo el pedido
    pub id_nodo: u8,
    /// id del usuario implicado
    pub id_cuenta: u32,
    /// id de la transaccion commiteada
    pub id_transaccion: u32,
    /// id dela cafetera del nodo
    pub id_cafetera: u8,
}

impl MensajeBytes for OkeyToCoordinator {
    fn new(id_nodo: u8, id_cuenta: u32, id_transaccion: u32, id_cafetera: u8) -> OkeyToCoordinator {
        OkeyToCoordinator {
            tipo_mensaje: Mensaje::OKEY.to_bytes(),
            id_nodo,
            id_cuenta,
            id_transaccion,
            id_cafetera,
        }
    }
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_id_transaccion(&self) -> u32 {
        self.id_transaccion
    }

    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Mensaje enviado por los nodos al coordinador luego de recibir un abort
pub struct OkeyAbortToCoordinator {
    /// tipo de mensaje (OkeyAbortToCoordinator)
    pub tipo_mensaje: u8,
    /// id del nodo que realizaba el pedido
    pub id_nodo: u8,
    /// id del usuario implicado
    pub id_cuenta: u32,
    /// id de la transaccion abortada
    pub id_transaccion: u32,
    /// id de la cafetera del nodo
    pub id_cafetera: u8,
}

impl MensajeBytes for OkeyAbortToCoordinator {
    fn new(
        id_nodo: u8,
        id_cuenta: u32,
        id_transaccion: u32,
        id_cafetera: u8,
    ) -> OkeyAbortToCoordinator {
        OkeyAbortToCoordinator {
            tipo_mensaje: Mensaje::OKEYABORT.to_bytes(),
            id_nodo,
            id_cuenta,
            id_transaccion,
            id_cafetera,
        }
    }
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_id_transaccion(&self) -> u32 {
        self.id_transaccion
    }

    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Mensaje abort
pub struct Abort {
    /// tipo de mensaje (abort)
    pub tipo_mensaje: u8,
    /// id del nodo que realizaba el pedido
    pub id_nodo: u8,
    /// id del usuario implicado
    pub id_cuenta: u32,
    /// id de la transaccion
    pub id_transaccion: u32,
    /// id de la cafetera del nodo
    pub id_cafetera: u8,
}

impl MensajeBytes for Abort {
    fn new(id_nodo: u8, id_cuenta: u32, id_transaccion: u32, id_cafetera: u8) -> Abort {
        Abort {
            tipo_mensaje: Mensaje::ABORT.to_bytes(),
            id_nodo,
            id_cuenta,
            id_transaccion,
            id_cafetera,
        }
    }
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_id_transaccion(&self) -> u32 {
        self.id_transaccion
    }

    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CommitType {
    SUMA = 0,
    RESTA = 1,
    UNKNOWN = 2,
}

impl CommitType {
    pub fn to_bytes(&self) -> u8 {
        match self {
            CommitType::SUMA => 0,
            CommitType::RESTA => 1,
            CommitType::UNKNOWN => 2,
        }
    }

    pub fn from_bytes(bytes: u8) -> CommitType {
        match bytes {
            0 => CommitType::SUMA,
            1 => CommitType::RESTA,
            _ => CommitType::UNKNOWN,
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Mensaje commit, enviado de un coordinador a sus nodos
pub struct Commit {
    /// tipo de mensaje (commit)
    pub tipo_mensaje: u8,
    /// id del nodo que realizo la operacion
    pub id_nodo: u8,
    /// id del usuario
    pub id_cuenta: u32,
    /// id de la transaccion commiteda
    pub id_transaccion: u32,
    /// tipo de operacion realizada
    pub tipo: CommitType,
    /// cantidad de creditos implicados
    pub cantidad: u32,
    /// id de la cafetera del nodo
    pub id_cafetera: u8,
}

impl Commit {
    pub fn new(
        id_nodo: u8,
        id_cuenta: u32,
        id_transaccion: u32,
        tipo: CommitType,
        cantidad: u32,
        id_cafetera: u8,
    ) -> Commit {
        Commit {
            tipo_mensaje: Mensaje::COMMIT.to_bytes(),
            id_nodo,
            id_cuenta,
            id_transaccion,
            tipo,
            cantidad,
            id_cafetera,
        }
    }

    pub fn set_to_string(&self) -> String {
        let mut result = String::new();
        let tipo_mensaje = self.get_tipo_mensaje().to_string();
        let id_nodo = self.get_id_nodo().to_string();
        let id_cuenta = self.get_id_cuenta().to_string();
        let id_transaccion = self.get_id_transaccion().to_string();
        let tipo = (self.tipo as u8).to_string();
        let cantidad = self.cantidad.to_string();
        let id_cafetera = self.id_cafetera.to_string();
        result.push_str(&tipo_mensaje);
        result.push('-');
        result.push_str(&id_nodo);
        result.push('-');
        result.push_str(&id_cuenta);
        result.push('-');
        result.push_str(&id_transaccion);
        result.push('-');
        result.push_str(&tipo);
        result.push('-');
        result.push_str(&cantidad);
        result.push('-');
        result.push_str(&id_cafetera);
        result
    }

    pub fn from_string(string: String) -> Commit {
        // split for "-" in string
        let bytes: Vec<String> = string.split('-').map(|x| x.to_string()).collect();

        let id_nodo: u8 = bytes[1].parse().expect("Error parsing id_nodo");
        let id_cuenta: u32 = bytes[2].parse().expect("Error parsing id_cuenta");
        let id_transaccion: u32 = bytes[3].parse().expect("Error parsing id_transaccion");
        let tipo = CommitType::from_bytes(bytes[4].parse().expect("Error parsing tipo"));
        let cantidad: u32 = bytes[5].parse().expect("Error parsing cantidad");
        let id_cafetera: u8 = bytes[6].parse().expect("Error parsing id_cafetera");

        Commit::new(
            id_nodo,
            id_cuenta,
            id_transaccion,
            tipo,
            cantidad,
            id_cafetera,
        )
    }

    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_id_transaccion(&self) -> u32 {
        self.id_transaccion
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
///Mensaje ping enviado de un nodo a un coordinador para validar coneccion
pub struct PingCord {
    /// tipo de mensaje (PingCord)
    pub tipo_mensaje: u8,
    /// id del nodo que envia ping
    pub id_nodo: u8,
    pub id_cuenta: u32,
    pub id_transaccion: u32,
    pub id_cafetera: u8,
}

impl MensajeBytes for PingCord {
    fn new(id_nodo: u8, id_cuenta: u32, id_transaccion: u32, id_cafetera: u8) -> PingCord {
        PingCord {
            tipo_mensaje: Mensaje::PING.to_bytes(),
            id_nodo,
            id_cuenta,
            id_transaccion,
            id_cafetera,
        }
    }
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_id_transaccion(&self) -> u32 {
        self.id_transaccion
    }

    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
}

// TODO: Agregar tests
