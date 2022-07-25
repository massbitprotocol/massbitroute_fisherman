provider "google-beta" {
    credentials = file("./project_key.json")
    project = "mbr-test-341307" //replace project name here
    region  = "us-central1" //replace region name here
    zone    = "us-central1-a"
}
