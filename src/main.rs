#![windows_subsystem = "windows"]

use iced::advanced::Application;
use pumpbin::Pumpbin;

fn main() -> iced::Result {
    Pumpbin::run(Pumpbin::settings())
}
