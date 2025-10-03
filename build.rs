use std::env;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use phf_codegen::Map;
use quote::quote;

/* This build script contains a "parser" for the PCI ID database.
 * "Parser" is in scare-quotes because it's really a line matcher with a small amount
 * of context needed for pairing nested entities (e.g. devices) with their parents (e.g. vendors).
 */

struct CgVendor {
    id: u16,
    name: String,
    devices: Vec<CgDevice>,
}

struct CgDevice {
    id: u16,
    name: String,
    subsystems: Vec<CgSubSystem>,
}

struct CgSubSystem {
    subvendor: u16,
    subdevice: u16,
    name: String,
}

struct CgClass {
    id: u8,
    name: String,
    subclasses: Vec<CgSubclass>,
}

struct CgSubclass {
    id: u8,
    name: String,
    prog_ifs: Vec<CgProgIf>,
}

pub struct CgProgIf {
    id: u8,
    name: String,
}

#[allow(clippy::redundant_field_names)]
fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    if update_ids().is_err() {
        println!("cargo:warning=Failed fetching pci ids, do you have internet connection ?... Using cached version");
    }
    let src_path = Path::new("pciids/pci.ids");
    let dest_path = Path::new(&out_dir).join("pci_ids.cg.rs");
    let input = {
        let f = fs::File::open(src_path).unwrap();
        BufReader::new(f)
    };
    let mut output = {
        let f = fs::File::create(dest_path).unwrap();
        BufWriter::new(f)
    };

    // Parser state.
    let mut curr_vendor: Option<CgVendor> = None;
    let mut curr_device_id = 0u16;
    let mut curr_class: Option<CgClass> = None;
    let mut curr_subclass_id = 0u8;

    let mut vendors = Map::new();
    let mut classes = Map::new();

    for line in input.lines() {
        let line = line.unwrap();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Ok((name, id)) = parser::vendor(&line) {
            // If there was a previous vendor, emit it.
            if let Some(vendor) = curr_vendor.take() {
                vendors.entry(vendor.id, &quote!(#vendor).to_string());
            }

            // Set our new vendor as the current vendor.
            curr_vendor = Some(CgVendor {
                id,
                name: name.into(),
                devices: vec![],
            });
        } else if let Ok((name, id)) = parser::device(&line) {
            // We should always have a current vendor; failure here indicates a malformed input.
            let curr_vendor = curr_vendor.as_mut().unwrap();
            curr_vendor.devices.push(CgDevice {
                id,
                name: name.into(),
                subsystems: vec![],
            });
            curr_device_id = id;
        } else if let Ok((name, (subvendor, subdevice))) = parser::subsystems(&line) {
            // We should always have a current vendor; failure here indicates a malformed input.
            // Similarly, our current vendor should always have a device corresponding
            // to the current device id.
            let curr_vendor = curr_vendor.as_mut().unwrap();
            let curr_device = curr_vendor
                .devices
                .iter_mut()
                .find(|d| d.id == curr_device_id)
                .unwrap();

            curr_device.subsystems.push(CgSubSystem {
                subvendor,
                subdevice,
                name: name.into(),
            });
        } else if let Ok((name, id)) = parser::class(&line) {
            // If there was a previous class, emit it.
            if let Some(class) = curr_class.take() {
                classes.entry(class.id, &quote!(#class).to_string());
            }

            // Set our new class as the current class.
            curr_class = Some(CgClass {
                id,
                name: name.into(),
                subclasses: vec![],
            });
        } else if let Ok((name, id)) = parser::subclass(&line) {
            // We should always have a current class; failure here indicates a malformed input.
            let curr_class = curr_class.as_mut().unwrap();
            curr_class.subclasses.push(CgSubclass {
                id,
                name: name.into(),
                prog_ifs: vec![],
            });
            curr_subclass_id = id;
        } else if let Ok((name, id)) = parser::prog_if(&line) {
            // We should always have a current class; failure here indicates a malformed input.
            // Similarly, our current class should always have a subclass corresponding
            // to the current subclass id.
            let curr_class = curr_class.as_mut().unwrap();
            let curr_subclass = curr_class
                .subclasses
                .iter_mut()
                .find(|d| d.id == curr_subclass_id)
                .unwrap();

            curr_subclass.prog_ifs.push(CgProgIf {
                id,
                name: name.into(),
            });
        } else {
            // TODO: Lots of other things that could be parsed out:
            // Language, dialect, country code, HID types, ...
            break;
        }
    }
    if let Some(vendor) = curr_vendor.take() {
        vendors.entry(vendor.id, &quote!(#vendor).to_string());
    }
    if let Some(class) = curr_class.take() {
        classes.entry(class.id, &quote!(#class).to_string());
    }

    writeln!(
        output,
        "static VENDORS: phf::Map<u16, Vendor> = {};",
        vendors.build()
    )
    .unwrap();

    writeln!(
        output,
        "static CLASSES: phf::Map<u8, Class> = {};",
        classes.build()
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=pciids/pci.ids");
}

mod parser {
    use std::num::ParseIntError;

    use nom::bytes::complete::{tag, take};
    use nom::character::complete::{hex_digit1, tab};
    use nom::combinator::{all_consuming, map_parser, map_res};
    use nom::sequence::{delimited, separated_pair, terminated};
    use nom::IResult;

    fn id<T, F>(size: usize, from_str_radix: F) -> impl Fn(&str) -> IResult<&str, T>
    where
        F: Fn(&str, u32) -> Result<T, ParseIntError>,
    {
        move |input| {
            map_res(map_parser(take(size), all_consuming(hex_digit1)), |input| {
                from_str_radix(input, 16)
            })(input)
        }
    }

    pub fn vendor(input: &str) -> IResult<&str, u16> {
        let id = id(4, u16::from_str_radix);
        terminated(id, tag("  "))(input)
    }

    pub fn device(input: &str) -> IResult<&str, u16> {
        let id = id(4, u16::from_str_radix);
        delimited(tab, id, tag("  "))(input)
    }

    pub fn subsystems(input: &str) -> IResult<&str, (u16, u16)> {
        let subvendor = id(4, u16::from_str_radix);
        let subdevice = id(4, u16::from_str_radix);
        let id = separated_pair(subvendor, tag(" "), subdevice);
        delimited(tag("\t\t"), id, tag("  "))(input)
    }

    pub fn class(input: &str) -> IResult<&str, u8> {
        let id = id(2, u8::from_str_radix);
        delimited(tag("C "), id, tag("  "))(input)
    }

    pub fn subclass(input: &str) -> IResult<&str, u8> {
        let id = id(2, u8::from_str_radix);
        delimited(tab, id, tag("  "))(input)
    }

    pub fn prog_if(input: &str) -> IResult<&str, u8> {
        let id = id(2, u8::from_str_radix);
        delimited(tag("\t\t"), id, tag("  "))(input)
    }
}

impl quote::ToTokens for CgVendor {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let CgVendor {
            id: vendor_id,
            name,
            devices,
        } = self;

        let devices = devices.iter().map(|CgDevice { id, name, subsystems }| {
            quote! {
                Device { vendor_id: #vendor_id, id: #id, name: #name, subsystems: &[#(#subsystems),*] }
            }
        });
        tokens.extend(quote! {
            Vendor { id: #vendor_id, name: #name, devices: &[#(#devices),*] }
        });
    }
}

impl quote::ToTokens for CgSubSystem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let CgSubSystem {
            subvendor,
            subdevice,
            name,
        } = self;
        tokens.extend(quote! {
            SubSystem { subvendor: #subvendor, subdevice: #subdevice, name: #name }
        });
    }
}

impl quote::ToTokens for CgClass {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let CgClass {
            id: class_id,
            name,
            subclasses,
        } = self;

        let subclasses = subclasses.iter().map(|CgSubclass { id, name, prog_ifs }| {
            quote! {
                Subclass { class_id: #class_id, id: #id, name: #name, prog_ifs: &[#(#prog_ifs),*] }
            }
        });
        tokens.extend(quote! {
            Class { id: #class_id, name: #name, subclasses: &[#(#subclasses),*] }
        })
    }
}

impl quote::ToTokens for CgProgIf {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let CgProgIf { id, name } = self;
        tokens.extend(quote! {
            ProgIf { id: #id, name: #name }
        });
    }
}

fn update_ids() -> Result<(), std::io::Error> {
    let status = std::process::Command::new("curl")
        .arg("https://raw.githubusercontent.com/pciutils/pciids/master/pci.ids")
        .arg("--create-dirs")
        .arg("--output")
        .arg("pciids/pci.ids")
        .spawn()?
        .wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error fetching pci data",
        ))
    }
}
