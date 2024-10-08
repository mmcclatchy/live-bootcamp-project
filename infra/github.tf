provider "github" {
  owner = var.github_username
  token = var.github_access_token
}

data "github_repository" "live_rust_bootcamp" {
  name = "live-bootcamp-project"
}

resource "random_password" "jwt_secret" {
  length  = 64
  special = false
}

resource "github_actions_secret" "admin_email" {
  repository      = data.github_repository.live_rust_bootcamp.name
  secret_name     = "ADMIN_EMAIL"
  plaintext_value = var.personal_email_address
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

resource "github_actions_secret" "jwt_secret" {
  repository      = data.github_repository.live_rust_bootcamp.name
  secret_name     = "JWT_SECRET"
  plaintext_value = random_password.jwt_secret.result
}

resource "github_actions_secret" "postgres_password" {
  repository      = data.github_repository.live_rust_bootcamp.name
  secret_name     = "POSTGRES_PASSWORD"
  plaintext_value = var.postgres_password
}

resource "github_actions_secret" "postmark_auth_token" {
  repository      = data.github_repository.live_rust_bootcamp.name
  secret_name     = "POSTMARK_AUTH_TOKEN"
  plaintext_value = var.postmark_auth_token
}

resource "github_actions_secret" "redis_password" {
  repository      = data.github_repository.live_rust_bootcamp.name
  secret_name     = "REDIS_PASSWORD"
  plaintext_value = var.redis_password
}

resource "github_actions_variable" "domain_name" {
  repository    = data.github_repository.live_rust_bootcamp.name
  variable_name = "DOMAIN_NAME"
  value         = local.full_domain_name
}

resource "github_actions_variable" "droplet_ip" {
  repository    = data.github_repository.live_rust_bootcamp.name
  variable_name = "DROPLET_IP"
  value         = digitalocean_droplet.monorepo.ipv4_address
}
