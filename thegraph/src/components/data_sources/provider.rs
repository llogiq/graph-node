use data::schema::Schema;
use futures::sync::mpsc::Receiver;
use serde_yaml;
use util::stream::StreamError;

/// Events emitted by [DataSourceProvider](trait.DataSourceProvider.html) implementations.
#[derive(Clone, Debug)]
pub enum DataSourceProviderEvent {
    /// A data source was added to the provider.
    DataSourceAdded(&'static str),
    /// A data source was removed from the provider.
    DataSourceRemoved(&'static str),
}

/// Schema-only events emitted by a [DataSourceProvider](trait.DataSourceProvider.html).
#[derive(Clone, Debug)]
pub enum SchemaEvent {
    /// A data source with a new schema was added.
    SchemaAdded(Schema),
    /// A data source with an existing schema was removed.
    SchemaRemoved(Schema),
}

/// Common trait for data source providers.
pub trait DataSourceProvider {
    /// Receiver from which others can read events emitted by the data source provider.
    /// Can only be called once. Any consecutive call will result in a StreamError.
    fn event_stream(&mut self) -> Result<Receiver<DataSourceProviderEvent>, StreamError>;

    /// Receiver from whith others can read schema-only events emitted by the data source provider.
    /// Can only be called once. Any consecutive call will result in a StreamError.
    fn schema_event_stream(&mut self) -> Result<Receiver<SchemaEvent>, StreamError>;
}

/// Common trait for data source definition schema.
pub trait DataSourceDefinition {
    // Deserialize serde_yaml into data source definition struct
    fn build_definition(definition: serde_yaml::Value) -> Self;

    // Extract graphql schema from data source definition struct
    fn extract_schema(&self) -> Schema;
}