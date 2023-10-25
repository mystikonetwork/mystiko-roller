use rusoto_sns::PublishInput;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct MonitorAlert<M> {
    pub error_message: String,
    #[builder(default)]
    pub topic_arn: Option<String>,
    #[builder(default, setter(skip))]
    _phantom: std::marker::PhantomData<M>,
}

pub trait IntoMessage<M> {
    fn into_message(self) -> M;
}

impl IntoMessage<PublishInput> for MonitorAlert<PublishInput> {
    fn into_message(self) -> PublishInput {
        PublishInput {
            message: self.error_message,
            topic_arn: self.topic_arn,
            ..Default::default()
        }
    }
}
