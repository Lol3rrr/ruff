# Ruff
A Rust based Scraper for Buff163 and other Marketplaces to export metrics to Prometheus

## Currency Conversion Rates
Can automatically load exchange rates from [OpenExchangeRates](https://openexchangerates.org/), which has a free tier that
is suitable. The rates are only loaded on startup and then once every day, as this is assumed to not change rapidly.

## Update Items at runtime
The items that should be tracked can be changed at runtime, by changing the configuration file that was used for the initial
configuration to reflect the new items and then sending a SIGHUP to the running process

## Marketplaces
- Buff163
- CSFloat
    - Enabled by providing an API_TOKEN using the environment variable `CSFLOAT_API_TOKEN`
