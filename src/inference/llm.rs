use crate::inference::engine::{InferErr, InferResp, VectorEncoding};
use crate::InferenceEngine;
use llm::{models::Llama, Model};
use std::{future::Future, sync::Arc};

impl VectorEncoding for LLMEncoding {}

pub struct LLMEncoding {
    value: Vec<(Vec<u8>, i32)>,
}

impl LLMEncoding {
    pub fn new(v: &Vec<(&[u8], i32)>) -> Self {
        let value: Vec<(Vec<u8>, i32)> = v.iter().map(|&(bytes, val)| (bytes.to_vec(), val)).collect();
        LLMEncoding { value }
    }

    pub fn get(&self) -> Vec<(&[u8], i32)> {
        self.value.iter().map(|(bytes, val)| (bytes.as_slice(), *val)).collect()
    }
}

#[derive(Clone)]
pub struct LlmInferenceEngine {
    model: Arc<Llama>,
}

impl InferenceEngine<LLMEncoding> for LlmInferenceEngine {

    fn new(model_path: String) -> Self {
        LlmInferenceEngine {
            model: Arc::new(
                llm::load::<Llama>(
                    std::path::Path::new(&model_path),
                    Default::default(),
                    llm::load_progress_callback_stdout,
                )
                .unwrap_or_else(|err| panic!("Failed to load model: {err}")),
            ),
        }
    }

    fn infer(
        &self,
        prompt: String,
    ) -> Box<dyn Future<Output = Result<InferResp, InferErr>> + Send> {
        let llama = self.model.clone();
        let future = {
            let mut session = llama.start_session(Default::default());
            let res = session.infer::<std::convert::Infallible>(
                &*llama,
                &mut rand::thread_rng(),
                &llm::InferenceRequest {
                    prompt: &prompt,
                    ..Default::default()
                },
                &mut Default::default(),
                |t| {
                    print!("{t}");
                    Ok(())
                },
            );

            let x = match res {
                Ok(inferenceStats) => Ok(InferResp {
                    result: inferenceStats.to_string(),
                }),
                Err(inferenceError) => Err(InferErr {
                    message: inferenceError.to_string(),
                }),
            };
            x
        };

        Box::new(async move { future })
    }

    fn encode(&self, document: String) -> LLMEncoding {
        let x = &self.model.vocabulary().tokenize(document.as_str(), false);
        match x {
            Ok(v) => LLMEncoding::new(&v),
            Err(_) => todo!(),
        }
    }
}
