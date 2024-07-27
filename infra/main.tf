terraform {
  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
    namecheap = {
      source  = "namecheap/namecheap"
      version = ">= 2.0.0"
    }
    github = {
      source  = "integrations/github"
      version = "~> 6.0"
    }
  }
}

provider "digitalocean" {
  token = var.do_token
}

provider "namecheap" {
  user_name = var.namecheap_username
  api_user  = var.namecheap_username
  api_key   = var.namecheap_api_key
}

resource "digitalocean_project" "bootcamp" {
  name    = "live-bootcamp"
  purpose = "Class project / Educational purposes"
}

resource "digitalocean_droplet" "monorepo" {
  image  = "ubuntu-24-04-x64"
  name   = "live-bootcamp-monorepo"
  region = "nyc1"
  size   = "s-1vcpu-512mb-10gb"

  user_data = templatefile("${path.module}/user_data.tftpl", {
    root_password    = var.droplet_root_password
    email_address    = var.personal_email_address
    full_domain_name = "${var.subdomain_name}.${var.domain_name}"
  })

}

resource "digitalocean_project_resources" "project_resources" {
  project   = digitalocean_project.bootcamp.id
  resources = [digitalocean_droplet.monorepo.urn]
}

resource "namecheap_domain_records" "personal" {
  domain = "markmcclatchy.com"

  record {
    hostname = "rust-bc"
    type     = "A"
    address  = digitalocean_droplet.monorepo.ipv4_address
  }

  record {
    hostname = "www"
    type     = "CNAME"
    address  = "mmcclatchy.github.io."
  }

  record {
    hostname = "@"
    type     = "TXT"
    address  = var.zoho_record_address
  }

  record {
    hostname = "@"
    type     = "A"
    address  = "185.199.108.153"
  }

  record {
    hostname = "@"
    type     = "A"
    address  = "185.199.109.153"
  }

  record {
    hostname = "@"
    type     = "A"
    address  = "185.199.110.153"
  }

  record {
    hostname = "@"
    type     = "A"
    address  = "185.199.111.153"
  }
}
