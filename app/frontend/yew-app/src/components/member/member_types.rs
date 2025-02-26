#[derive(PartialEq, Clone, Debug)]
pub struct Member {
  pub name: String,
  pub role: String,
  pub description: String,
  pub main_image: String,
  pub sub_images: Vec<String>,
}