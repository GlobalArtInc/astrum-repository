# CocoaPods

CocoaPods repositories are proxies of a CocoaPods CDN source, such as trunk at `https://cdn.cocoapods.org/`.
Name the repository as the source of the Podfile:

```ruby
source 'https://astrum.example.com/repository/cocoapods-proxy/'

target 'App' do
  pod 'Alamofire', '~> 5.9'
end
```

or add it once to the local spec repositories, for `pod spec cat` and `pod repo update`:

```shell
pod repo add-cdn astrum https://astrum.example.com/repository/cocoapods-proxy/
```

CocoaPods signs in only through `~/.netrc`, which `pod` and the `curl` it downloads archives with both read:

```text
machine astrum.example.com
login ci-reader
password <password or access token>
```

A podspec whose source is an HTTP(S) archive is served with that address pointed at
`pods/<pod>/<version>/<file>` of the repository, so the archive is downloaded and cached through it; pods
with a Git source are still cloned from their Git server. Because the served podspecs differ from those of
trunk, the `SPEC CHECKSUMS` in `Podfile.lock` change when a project moves between trunk and the proxy.
