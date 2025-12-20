

const DEFAULT_SCONNY_CONFIG_FILE: &str = "sconny_config.toml";

pub enum llm_services {
    OpenAI,
    Gemini
}

pub enum scy_os {
    Windows,
    Linux
}


pub struct sconny_setting {
    pub llm_service: llm_services,
    pub scy_os: scy_os
}


impl sconny_setting {
    pub fn new() -> sconny_setting {
        sconny_setting {
            llm_service: llm_services::OpenAI,
            scy_os: scy_os::Linux
        }
    }

    pub fn load_setting(&mut self) {
        // Placeholder for loading settings from a file or environment
        // For now, we just set some default values

        // Parse Setting File Here 
        

        self.llm_service = llm_services::OpenAI;
        self.scy_os = scy_os::Linux;
    }
}