variable "do_token" {}
variable "droplet_root_password" {}

terraform {
  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }
}

provider "digitalocean" {
  token = var.do_token
}

data "digitalocean_vpc" "default" {
  name = "default-nyc1"
}

# resource "digitalocean_ssh_key" "personal_ssh" {
#   name       = "Terraform Example"
#   public_key = file("/Users/markmcclatchy/.ssh/id_rsa.pub")
# }

resource "digitalocean_project" "bootcamp" {
  name    = "live-bootcamp"
  purpose = "Class project / Educational purposes"
}

resource "digitalocean_droplet" "monorepo" {
  image  = "ubuntu-24-04-x64"
  name   = "live-bootcamp-monorepo"
  region = "nyc1"
  size   = "s-1vcpu-512mb-10gb"
  # ssh_keys = [digitalocean_ssh_key.personal_ssh.id]

  user_data = templatefile("${path.module}/user_data.tftpl", {
    root_password = var.droplet_root_password
  })

}

resource "digitalocean_project_resources" "project_resources" {
  project   = digitalocean_project.bootcamp.id
  resources = [digitalocean_droplet.monorepo.urn]
}

resource "digitalocean_domain" "personal" {
  name = "markmcclatchy.com"
}

resource "digitalocean_record" "rust_bc" {
  domain = digitalocean_domain.personal.name
  type   = "A"
  name   = "rust-bc"
  value  = digitalocean_droplet.monorepo.ipv4_address
}
