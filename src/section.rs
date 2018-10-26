use context::Context;
use failure::Error;
use serde_json;

#[derive(Debug)]
pub enum Exec {
    SQL,
    Shell,
    Python,
}

#[derive(Debug)]
pub struct Section {
    pub include: Option<String>,
    pub reference: String,
    pub format: String,
    pub process: Option<Exec>,
    pub body: String,
}

impl Section {
    pub fn from(header: &str, body: &str) -> Result<Section, Error> {
        Ok(Section {
            include: None,
            reference: header.into(),
            format: "json".into(),
            process: None,
            body: body.into(),
        })
    }

    pub fn parse(txt: &str) -> Result<Vec<Section>, Error> {
        let txt = "\n".to_owned() + txt;
        println!("txt: {}", &txt);
        let mut sections = vec![];
        for part in txt.split("\n--").skip(1) {
            let part = part.to_owned() + "\n";
            let split = part.splitn(2, '\n').collect::<Vec<&str>>();
            let (header, body) = (split[0], split[1]);
            sections.push(Section::from(header, body)?);
        }
        Ok(sections)
    }

    pub fn apply<T>(&self, value: serde_json::Value, ctx: &T) -> Result<serde_json::Value, Error>
    where
        T: Context,
    {
        Ok(serde_json::from_str(&self.body)?)
    }
}
