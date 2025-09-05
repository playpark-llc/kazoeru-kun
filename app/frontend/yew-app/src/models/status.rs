#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Dirty,
    Unclean,
    Clean,
    Immaculate,
    Spotless,
}

impl Status {
    pub fn current_target(&self) -> u32 {
        match self {
            Status::Dirty => 100_000,
            Status::Unclean => 100_000,
            Status::Clean => 100_000,
            Status::Immaculate => 100_000,
            Status::Spotless => 100_000,
        }
    }

    pub fn image_url(&self) -> &'static str {
        match self {
            Status::Dirty => "/images/status/dirty.png",
            Status::Unclean => "/images/status/unclean.png",
            Status::Clean => "/images/status/clean.png",
            Status::Immaculate => "/images/status/immaculate.png",
            Status::Spotless => "/images/status/spotless.png",
        }
    }
}

impl From<&str> for Status {
    fn from(s: &str) -> Self {
        match s {
            "DIRTY" => Status::Dirty,
            "UNCLEAN" => Status::Unclean,
            "CLEAN" => Status::Clean,
            "IMMACULATE" => Status::Immaculate,
            "SPOTLESS" => Status::Spotless,
            _ => Status::Clean,
        }
    }
}