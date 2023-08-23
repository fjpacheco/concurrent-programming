use actix::Message;

#[derive(Debug, Clone, Copy)]
pub enum MensajeBully {
    OKEY = 0,
    ELECTION,
    COORDINATOR,
    PING,
    PINGCORD,
    DESCONOCIDO,
}

impl MensajeBully {
    pub fn from_bytes(byte: u8) -> MensajeBully {
        match byte {
            0_u8 => MensajeBully::OKEY,
            1_u8 => MensajeBully::ELECTION,
            2_u8 => MensajeBully::COORDINATOR,
            3_u8 => MensajeBully::PING,
            4_u8 => MensajeBully::PINGCORD,
            _ => MensajeBully::DESCONOCIDO,
        }
    }
}

pub trait MensajeBullyBytes {
    fn new(id_nodo: u8) -> Self;
    fn get_id_nodo(&self) -> u8;
    fn get_tipo_mensaje(&self) -> u8;

    fn to_bytes(&self) -> Vec<u8> {
        vec![self.get_tipo_mensaje(), self.get_id_nodo()]
    }

    fn from_bytes(bytes: Vec<u8>) -> Self
    where
        Self: Sized,
    {
        Self::new(bytes[1])
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct OkeyBully {
    pub tipo_mensaje: u8,
    pub id_nodo: u8,
}

impl MensajeBullyBytes for OkeyBully {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }

    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }

    fn new(id_nodo: u8) -> OkeyBully {
        OkeyBully {
            tipo_mensaje: MensajeBully::OKEY as u8,
            id_nodo,
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Election {
    pub tipo_mensaje: u8,
    pub id_nodo: u8,
}

impl MensajeBullyBytes for Election {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }

    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }

    fn new(id_nodo: u8) -> Election {
        Election {
            tipo_mensaje: MensajeBully::ELECTION as u8,
            id_nodo,
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Coordinator {
    pub tipo_mensaje: u8,
    pub id_nodo: u8,
}

impl MensajeBullyBytes for Coordinator {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }

    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }

    fn new(id_nodo: u8) -> Coordinator {
        Coordinator {
            tipo_mensaje: MensajeBully::COORDINATOR as u8,
            id_nodo,
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Ping {
    pub tipo_mensaje: u8,
    pub id_nodo: u8,
}

impl MensajeBullyBytes for Ping {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }

    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }

    fn new(id_nodo: u8) -> Ping {
        Ping {
            tipo_mensaje: MensajeBully::PING as u8,
            id_nodo,
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct PingCord {
    pub tipo_mensaje: u8,
    pub id_nodo: u8,
}

impl MensajeBullyBytes for PingCord {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }

    fn get_id_nodo(&self) -> u8 {
        self.id_nodo
    }

    fn new(id_nodo: u8) -> PingCord {
        PingCord {
            tipo_mensaje: MensajeBully::PINGCORD as u8,
            id_nodo,
        }
    }
}
