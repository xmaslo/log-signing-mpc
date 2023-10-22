#[derive(Clone, Debug)]
pub struct MPCconfig {
    server_id: u16,
    threshold: u16,
    number_of_parties: u16,
}

impl MPCconfig {
    pub fn new(si: u16, t: u16, n: u16) -> MPCconfig {
        MPCconfig{
            server_id: si,
            threshold: t,
            number_of_parties: n
        }
    }

    pub fn server_id(&self) -> u16 {
        self.server_id
    }
    pub fn threshold(&self) -> u16 {
        self.threshold
    }
    pub fn number_of_parties(&self) -> u16 {
        self.number_of_parties
    }
}