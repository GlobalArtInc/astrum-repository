# RubyGems

Activate the **RubyGems API Key Realm** to publish. `gem signin` asks for the user name and password
and stores the user's API key for the repository:

```shell
gem signin --host https://astrum.example.com/repository/rubygems-hosted/
gem push mylib-1.0.0.gem --host https://astrum.example.com/repository/rubygems-hosted/
gem yank mylib -v 1.0.0 --host https://astrum.example.com/repository/rubygems-hosted/
```

To install, use a group (or a proxy of rubygems.org) as the source, with credentials in the URL or in
Bundler's configuration (keyed by host name):

```shell
gem install mylib --clear-sources --source https://astrum.example.com/repository/rubygems-group/
bundle config set --global astrum.example.com "ci-reader:$ASTRUM_PASSWORD"
```

and in the `Gemfile`: `source "https://astrum.example.com/repository/rubygems-group/"`. To use the
repository as a mirror of rubygems.org run
`bundle config set --global mirror.https://rubygems.org https://astrum.example.com/repository/rubygems-group/`.
