<!-- PROJECT SHIELDS -->

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![GPLv3 License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/lnenad/Requestor">
    <img src="requestor-logo-wb.png" alt="Logo" width="120" height="120">
  </a>

  <h3 align="center">Requestor</h3>

  <p align="center">
    Lightweight API client
    <br />
    <a href="https://github.com/lnenad/Requestor"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/lnenad/Requestor">View Demo</a>
    ·
    <a href="https://github.com/lnenad/Requestor/issues">Report Bug</a>
    ·
    <a href="https://github.com/lnenad/Requestor/issues">Request Feature</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
      <ul>
        <li><a href="#features">Features</a></li>
      </ul>
    </li>
    <li>
      <a href="#development">Development</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->

## About The Project

[![Product Name Screen Shot][product-screenshot]](https://requestor.dev)

The goal of this is to have a free, open source alternative to resource intensive usually electron based application lifeforms. It's pretty barebones right now but I plan on adding features as time goes by. It's written in Rust so it should be lightweight.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Built With

- egui
- eframe
- ehttp

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->

## Features

The project is an attempt to create a lightweight API testing tool to counter the approach that current offering of costly, heavyweight tools. No subscriptions are planned, no cloud features. Simplicity is a priority and avoiding unnecessary feature bloat.

### Goals

- To be **lightweight**. This means startup time of the program is low, memory usage is low, processor usage is low.
- To stay **simple to use**. The happiest flow of opening the program, inputting the url, send the request should be as straightforward as possible.
- Stay **subscription free, local and open source**.

### Non-goals

- **Feature bloat**. Current offerings are very complex, for a small percentage of users this makes sense, this is not the goal for Requestor.
- **Cloud sync**. Having some sort of instant sync across multiple machines seems unnecessary for a huge percentage of people using API testing tools.

### Feature support

- **Tab support**. You have a huge screen? Great, you can split the main window into multiple tabbed layouts and speed up testing of different scenarios.
- **Environment support**. A simple key-value json file that can be loaded to provide an easy way to load secrets/fixed values across multiple requests.

### Environment setup

To use environments you need to have a simple key/value `json` file setup.

Example contents:

```json
{
  "url": "https://httpbin.org",
  "qs": "querystringvalue",
  "secret": "authheadervalue"
}
```

To use these values inside Requestor you need to load the file by clicking on the "Environment" dropdown within a tab and selecting "Load". The contents of the file will be read and stored in local app cache. If you change the file you can reload the contents by clicking on the 🔁 icon located in the upper right corner of the tab. After the file has been loaded you can preview the values by clicking on the ✅ icon located in the upper right corner of the tab.

After everything is ready you can use the curly-brace syntax, `{key}`, to inject the environment values into the inputs. Currently evaluated inputs are:

- url
- querystring keys and values
- header keys and values

If you set the url to `{url}/get` and perform the request, the request will be sent to `https://httpbin.org/get` as per loaded environment values. If the value is not set in the environment it will not be replaced.

## Development

To get a local copy up and running follow these simple example steps.

### Prerequisites

Have `cargo` and `Rust` setup on your computer.

### Installation

To use on Linux, first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

After that you should be able to use `cargo` to run and build.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- USAGE EXAMPLES -->

## Running the project

To run the project locally just execute `cargo run`.

## Building the project

To build the project execute `cargo build`.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ROADMAP -->

## Roadmap

- [ ] Add a nice preview and json formatting options
- [x] Add environment configuration
- [ ] Add project configuration
- [ ] Add projects that contain saved requests

See the [open issues](https://github.com/lnenad/Requestor/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTRIBUTING -->

## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- LICENSE -->

## License

Distributed under the GPLv3 License. See `LICENSE.txt` for more information.

Project Link: [https://github.com/lnenad/Requestor](https://github.com/lnenad/Requestor)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[contributors-shield]: https://img.shields.io/github/contributors/lnenad/Requestor?style=for-the-badge
[contributors-url]: https://github.com/lnenad/Requestor/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/lnenad/Requestor?style=for-the-badge
[forks-url]: https://github.com/lnenad/Requestor/network/members
[stars-shield]: https://img.shields.io/github/stars/lnenad/Requestor?style=for-the-badge
[stars-url]: https://github.com/lnenad/Requestor/stargazers
[issues-shield]: https://img.shields.io/github/issues/lnenad/Requestor?style=for-the-badge
[issues-url]: https://github.com/lnenad/Requestor/issues
[license-shield]: https://img.shields.io/github/license/lnenad/Requestor?style=for-the-badge
[license-url]: https://github.com/lnenad/Requestor/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/nenad-lukic-6a9b724b
[product-screenshot]: screenshot.png
