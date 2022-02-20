use actix_web::{web::Path};
use actix_web::HttpResponse;
use serde::Serialize;
use mongodb::{
    bson::doc,
    sync::{
        Cursor,
        Database,
        Collection},
    error::Error};

use crate::APPLICATION_JSON;

pub trait Minable<T> {
    fn to_min(&self)->T;
}

pub trait Insertable<T>{
    fn obj_entry_or_insert(self,dict: T) -> T;
}

pub trait Gettable<Info,Minimal>{
    fn get(&self, k: Path<Info>) -> Option<Minimal>;
}


#[derive(Clone)]
pub struct APIEndpointContainer<Base,DataContainer: Clone, Minimal: Clone, Info: Clone>{
    data: DataContainer,
    base: Base,
    min: Minimal,
    info: Info
}


impl<Minimal: Minable<Minimal> + std::default::Default +  Serialize + Clone,
     DataContainer: Clone + Gettable<Info,Minimal>,
     Info: Clone
     > APIEndpointContainer<Minimal,DataContainer,Minimal, Info>{

    pub fn list(self, path: Path<Info>) -> HttpResponse{


        match self.data.get(path){
            Some(data) =>{
                HttpResponse::Ok()
                .content_type(APPLICATION_JSON)
                .json(data)
            },
            None => HttpResponse::NotFound().content_type(APPLICATION_JSON).json(ErrorMessage{error:"No Match Found".to_string()})
        }    
    }
}

impl<
    Minimal:  Minable<Minimal> + Clone + Default + Insertable<DataContainer> ,
    T: Minable<Minimal> + serde::de::DeserializeOwned + Unpin + std::marker::Send + Sync + std::fmt::Debug,
    DataContainer: Clone+Default,
    Info: Clone + Default  >
    APIEndpointContainer<T,DataContainer,Minimal, Info>{
    pub fn new(endpoint: &str, db: &Database)-> APIEndpointContainer<Minimal,DataContainer,Minimal, Info>{
        println!("{}",endpoint);
        let mini: Minimal = std::default::Default::default();
        let blank_data: DataContainer = std::default::Default::default();
        let info = std::default::Default::default();
        let mut obj: APIEndpointContainer<Minimal, DataContainer, Minimal, Info> = APIEndpointContainer { data: blank_data, min: mini.clone(), base:mini, info:info };

        let collection: Collection<T> = db.collection::<T>(endpoint);

        let prog_count = collection.count_documents(doc! {}, None).unwrap();

        let items: Result<Cursor<T>,Error> = collection.find(doc! {}, None);
    
        match items {
            Ok(j) => {
                let prog = indicatif::ProgressBar::new(prog_count);
                for record in j{
                    prog.inc(1);
                    match record{
                        Ok(rec) => {
                            let min: Minimal = rec.to_min();
                            obj.data = min.obj_entry_or_insert(obj.data);
                            // obj.data.entry(min.mongo_match_id.to_owned())
                            //         .or_insert(HashMap::new())
                            //             .entry(min.account_id.clone())
                            //             .or_insert(HashMap::new())
                            //                 .insert(min._d.clone(), min);
                        },
                        Err(k) => println!("{:?}",k)
                    };
            }
            },
            Err(k) => println!("{:?}",k)
        }

        obj
    }
}



#[derive(Serialize)]
struct ErrorMessage{
 error: String
}

