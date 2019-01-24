use csv;

pub fn csv_to_list_list_string(content: &str) -> Vec<serde_json::Value> {
    let mut output: Vec<serde_json::Value> = Vec::new();
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    
    if let Ok(result) = reader.headers() {
        let mut serde_record: Vec<serde_json::Value> = Vec::new();
        for data in result.iter() {
            let value = serde_json::Value::String(data.to_owned());
            serde_record.push(value);
        }
        let serde_record = serde_json::Value::Array(serde_record);
        output.push(serde_record);
    };
    
    let mut iterator = reader.records();
    while let Some(result) = iterator.next() {
        for record in result.iter() {
            let mut serde_record: Vec<serde_json::Value> = Vec::new();
            for data in record.iter() {
                let value = serde_json::Value::String(data.to_owned());
                serde_record.push(value);
            }
            let serde_record = serde_json::Value::Array(serde_record);
            output.push(serde_record);
        }
    }
    return output;
}
