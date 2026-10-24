# Hugging Face

A Hugging Face proxy caches models, datasets and spaces of the Hugging Face Hub; its remote URL is
`https://huggingface.co/`. `huggingface_hub`, `transformers`, `datasets`, the `hf` command and every
other tool that honours `HF_ENDPOINT` download through it:

```shell
export HF_ENDPOINT=https://ci-reader:${ASTRUM_TOKEN}@astrum.example.com/repository/huggingface-proxy
export HF_HUB_DOWNLOAD_TIMEOUT=600
hf download openai-community/gpt2 config.json
python -c 'from huggingface_hub import snapshot_download; snapshot_download("openai-community/gpt2")'
```

The proxy fetches a file completely before it answers, so the first download of a large file can
take longer than the ten seconds clients wait by default; `HF_HUB_DOWNLOAD_TIMEOUT` raises that limit.
Hub clients only send Hugging Face tokens on their own, so the user name and password or access
token of this server go into `HF_ENDPOINT` as above. For gated and private repositories, put a
Hugging Face access token into the proxy's HTTP authentication as a bearer token. The proxy sends it
to the Hub only, never to the storage the Hub redirects large files to.

A branch or tag is resolved to its commit through the revision information, which is refreshed after
the metadata max age. Files are cached by commit and never refreshed, so a pinned commit is served
from the cache even while the Hub is unreachable, and a file that did not change between commits is
stored once. Search finds a commit by the repository id (the component name), the commit (the
version), `huggingface.revision` (a branch or tag that pointed at it) and `huggingface.file`.

Uploads, the Xet protocol (clients fall back to plain HTTP downloads) and searching the Hub
(`/api/models?search=`) are not supported.
