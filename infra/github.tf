provider "github" {
  owner = var.github_username
  token = var.github_access_token
}

data "github_repository" "live_rust_bootcamp" {
  name = "live-bootcamp-project"
}

resource "github_actions_secret" "docker_username" {
  repository      = data.github_repository.live_rust_bootcamp.name
  secret_name     = "DOCKER_USERNAME"
  plaintext_value = var.docker_username
}

resource "github_actions_secret" "docker_password" {
  repository      = data.github_repository.live_rust_bootcamp.name
  secret_name     = "DOCKER_PASSWORD"
  plaintext_value = var.docker_password
}

resource "github_actions_secret" "droplet_password" {
  repository      = data.github_repository.live_rust_bootcamp.name
  secret_name     = "DROPLET_PASSWORD"
  plaintext_value = var.droplet_root_password
}

resource "github_actions_variable" "droplet_ip" {
  repository    = data.github_repository.live_rust_bootcamp.name
  variable_name = "DROPLET_IP"
  value         = digitalocean_droplet.monorepo.ipv4_address
}
