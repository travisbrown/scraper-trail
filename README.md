# scraper-trail

[![Rust build status](https://img.shields.io/github/actions/workflow/status/travisbrown/scraper-trail/ci.yaml?branch=main)](https://github.com/travisbrown/scraper-trail/actions)
[![Coverage status](https://img.shields.io/codecov/c/github/travisbrown/scraper-trail/main.svg)](https://codecov.io/github/travisbrown/scraper-trail)

It's often useful to be able to save detailed information about requests and responses while developing a web scraper,
especially early in development, or for small projects.

## Context

This library was created to abstract common functionality that we had implemented in several projects.
You can see examples of its use in [this Rust library][app-store-access] for accessing and storing
data from the Google and Apple app stores, and in this [Meta Ads Archive client][meta-ads-scraper].
We are also using it in a couple of other contexts where the client software is not currently open source.

## License

This software is licensed under the [GNU General Public License v3.0][gpl-v3] (GPL-3.0).

[app-store-access]: https://github.com/travisbrown/app-store-access
[gpl-v3]: https://www.gnu.org/licenses/gpl-3.0.en.html
[meta-ads-scraper]: https://github.com/travisbrown/meta-ads-scraper
