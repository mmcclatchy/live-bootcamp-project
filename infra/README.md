# Terraform Managed Infrastructure

## Update .gitignore

```txt
**/target
**/.vscode/
.env
*.tfvars
*.auto.tfvars
**/.terraform/
.terraform.lock.hcl
*.backup
*.tfstate

```

- This can be put in the root level of the project
- If you prefer separate `.gitignore` files, then just keep `*.tfvars` down in the `infra/.gitignore` file

## Code changes

- In order for the paths to work correctly, changes to the `src` paths needed to be made
  - Instead of using local paths, absolute paths need to be used
  - Do this for all `<img src="...">`
    - From:

        ```html
        <img src="/assets/lgr_logo.png" alt="" width="25" height="25" class="d-inline-block align-text-top">
        ```

    - To:

        ```html
        <img src="/app/assets/lgr_logo.png" alt="" width="25" height="25" class="d-inline-block align-text-top">
        ```

  - `app-service/assets/app.js`
    - Use the absolute path for `protectImg.src`:

      ```js
      protectImg.src = "/app/assets/default.jpg";
      ```

  - `app-service/main.rs`
    - Update the `address` envar key:

      ```rust
      let mut address = env::var("AUTH_SERVICE_URL").unwrap_or("localhost/auth".to_owned());
      ```

  - `compose.yml`

    ```yml
    services:
      app-service:
        environment:
          AUTH_SERVICE_URL: {domain-with-subdomain}/auth
    ```

  - `compose.override.yml`

    ```yml
    services:
      app-service:
        environment:
          AUTH_SERVICE_URL: localhost/auth
    ```

## Using Terraform locally

- [Install Terraform](https://developer.hashicorp.com/terraform/tutorials/aws-get-started/install-cli)
- `cd` to your the `infra` directory or wherever `main.tf` resides
- `terraform init`
- Read below and perform any needed `terraform import` commands
- `terraform plan`
  - This safely checks what will change without making any changes
- Check the plan to see if everything looks right
  - What is being created, modified, and destroyed?
- `terraform apply`
  - This will create/change/delete resources
  - If any errors occur, there is no rollback
    - If there are 10 changes and an error occurs on change 5, the first 4 will remain changed
- A quick note about terraform resources, if you are not familiar:
  - Example:

    ```terraform
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

    data "github_repository" "live_rust_bootcamp" {
        name = "live-bootcamp-project"
    }
    ```

  - `resource` is a keyword of Terraform.
    - This declares that something will be managed by Terraform
  - `data` is another keyword of Terraform
    - This is read-only
    - Will not manage/change/delete something from the provider
  - The second position (first quoted string) is the name of the `resource` or `data` to be used by the provider
  - The third position (second quoted string) is an arbitrary name given the the `resource` or `data`
    - Names must be unique among a specific `resource` or `data` type
    - Names can be shared across different `resource` or `data` types
  - Information that pertains to the `resource`, `data`, or `variable` types can be accessed like:
    - `resource`
      - `{resource_type}.{arbitrary_given_name}.{property_name}`
      - `digitalocean_droplet.monorepo.ipv4_address`
    - `data`
      - `data.{data_type}.{arbitrary_given_name}.{property_name}`
      - `data.github_repository.live_rust_bootcamp.name`
    - `variable`
      - `var.{variable_name}`
      - `var.droplet_root_password`

## variables.tf

- These are variables that require being set within a `secrets.auto.tfvars`
  - `secrets` in this filename is arbitrary and can be named anything
  - `.auto.tfvars` is required
  - This will automatically define the variables in `variables.tf` with the values in `secrets.auto.tfvars`
- `.auto.tfvars` files are ignored by .gitignore
- The format of `secrets.auto.tfvars`:

    ```terraform
    do_token="..."
    droplet_root_password="..."
    namecheap_username="..."
    namecheap_api_key="..."
    zoho_record_address="..."
    github_username="..."
    github_access_token="..."
    docker_username="..."
    docker_password="..."
    domain_name="..."
    subdomain_name="..."
    personal_email_address="..."
    ```

## main.tf

### `terraform` block

- We declare what providers we are using within `required_providers`

  - `digitalocean`
  - `github`
  - `namecheap`
    - This one is for me specifically since the domain I own was purchased at namecheap.com
    - Most notable websites to buy domains will likely have similar implementations to this
      - google:  terraform your-domain-website
      - Terraform documentation is rather good, so it should be fairly straight forward

### `provider` blocks

- This is how we authenticate with the provider
- Typically api keys are created on your provider account page

### Digital Ocean

[Digital Ocean Terraform Docs](https://registry.terraform.io/providers/digitalocean/digitalocean/latest/docs)

#### `digitalocean_project`

- [digitalocean_project Docs](https://registry.terraform.io/providers/digitalocean/digitalocean/latest/docs/resources/project)
- This is the project we first created.
- Import this into your terraform state with:

  ```bash
  doctl projects list
  terraform import digitalocean_project.bootcamp {project-id}
  ```

#### `digitalocean_droplet`

- [digitalocean_droplet Docs](https://registry.terraform.io/providers/digitalocean/digitalocean/latest/docs/resources/droplet)
- Import this droplet into terraform state with:
- The `size` in this droplet is configured to use the smallest/cheapest server
  - `size   = "s-1vcpu-512mb-10gb"`
  - You can vertically scale this by choosing from the various slugs

    ```bash
    curl -X GET \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $YOUR_DIGITALOCEAN_API_TOKEN" \
        "https://api.digitalocean.com/v2/sizes"
    ```

  ```bash
  doctl compute droplet list
  terraform import digitalocean_droplet.monorepo {droplet-id}
  ```

#### Domain Records

- The `namecheap_domain_records` resource is particular to me because my domain was purchased through NameCheap
- If a domain was created with DigitalOcean, then these resources will be relevant to you
  - [digitalocean_domain Docs](https://registry.terraform.io/providers/digitalocean/digitalocean/latest/docs/resources/domain)
  - [digitalocean_record Docs](https://registry.terraform.io/providers/digitalocean/digitalocean/latest/docs/resources/record)
  - A good example of the terraform implementation is provided in the `digitalocean_record` docs

## github.tf

- [GitHub Terraform Docs](https://registry.terraform.io/providers/integrations/github/latest/docs)
- This will manage your GitHub Action Secrets and Variables
- A change to these values will always replace the secret
  - Importing these is not very helpful. It's easier to delete the secrets and let terraform create them

## user_data.tftpl

- This is used to configure and setup the Production Droplet Server
- This is a Terraform Template generates a script that will only be run when a Droplet is created
- The script:
  - Installs packages
  - Sets environment variables
  - Generates an SSL Certificate
  - Configures Nginx (`/etc/nginx/sites-available/default` file)
    - Sets the ports and paths being listened to
    - Routes requests to the correct service
    - Implements the SSL Certificate so that the URL routes to the right service
    - Permits `assets` to be used by the `app-service` and `auth-service` webpages
- Variables can be passed into this template by using this format: `${variable_name}`
- Provide the value for these variables by using `templatefile(path, variable_map)`

    ```terraform
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
    ```

## local/nginx.local.conf

- This is used to configure and setup the Local Nginx Container
- In `compose.override.yml` the `nginx` container is configured
  - This is needed to mirror the behavior of the Production Droplet Server in a simple way
- This will write to the `/etc/nginx/nginx.conf` file
