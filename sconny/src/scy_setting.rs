

const DEFAULT_SCONNY_CONFIG_FILE: &str = "sconny_config.toml";

pub enum LlmServices {
    OpenAI,
    Gemini
}

pub enum ScyOs {
    Windows,
    Linux
}


pub struct SconnySetting {
    pub llm_service: LlmServices,
    pub ScyOs: ScyOs
}


impl SconnySetting {
    pub fn new() -> SconnySetting {
        SconnySetting {
            llm_service: LlmServices::OpenAI,
            ScyOs: ScyOs::Linux
        }
    }

    pub fn load_setting(&mut self) {
        // Placeholder for loading settings from a file or environment
        // For now, we just set some default values

        // Parse Setting File Here 


        self.llm_service = LlmServices::OpenAI;
        self.ScyOs = ScyOs::Linux;
    }
}