#[macro_use] extern crate packed_struct_codegen;
mod arc;
use arc::*;
use std::path::PathBuf;
use structopt::StructOpt;
use std::collections::HashMap;
use binwrite::BinWrite;
use std::fs::File;

#[derive(StructOpt)]
struct Args {
    arc: PathBuf,
    out_file: PathBuf
}

fn use_arc(internal: ArcInternal, out_path: &PathBuf) {
    let matches = 
        internal.file_infos_v2
            .iter()
            .filter_map(|file_info: &FileInfo2|{
                let path_index = file_info.hash_index;
                let path: FileInformationPath = internal.file_info_paths[path_index as usize];
                let parent = path.parent;
                let full_path_hash = path.path.hash40();

                let parent_hash = parent.hash40();
                if internal.dir_hash_to_index.binary_search_by_key(&parent_hash, |h| h.hash40()).is_err() {
                    Some((parent_hash, full_path_hash))
                } else {
                    None
                }
            });

    let mut dirs = HashMap::<u64, Vec<u64>>::new();
    for (parent_hash, full_path_hash) in matches {
        match dirs.get_mut(&parent_hash) {
            Some(dir) => {
                dir.push(full_path_hash);
            }
            None =>  {
                dirs.insert(parent_hash, vec![ full_path_hash ]);
            }
        }
    }

    (
        dirs.len() as u64,
        dirs.into_iter()
            .map(|(parent_hash, children)|{
                (
                    children.len() as u64,
                    parent_hash,
                    children
                )
            })
            .collect::<Vec<_>>()
    ).write(&mut File::create(out_path).unwrap()).unwrap();
}

fn main() {
    let args = Args::from_args();

    Arc::open_and_use(&args.arc, |_, b| use_arc(b, &args.out_file)).unwrap();
}
