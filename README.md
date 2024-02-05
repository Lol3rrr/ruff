# Ruff
A Rust based Scraper for Buff163 to export metrics to Prometheus

## Currency Conversion Rates
Can automatically load exchange rates from [OpenExchangeRates](https://openexchangerates.org/), which has a free tier that
is suitable. The rates are only loaded on startup and then once every day, as this is assumed to not change rapidly.
