# Image signatures

A Docker repository can refuse images that no trusted key has signed. The policy of the repository names
the trusted keys and a mode. With the mode `block`, a manifest that is not signed by one of those keys is
refused; with `report` it is served and the result is recorded.

This page describes the signed-only gate as implemented in the version that is under review. Details may
change before it is released.

## What is verified

- **cosign key-based signatures**, in the two forms cosign writes:
  - a simple-signing layer (`application/vnd.dev.cosign.simplesigning.v1+json`) in a signature manifest,
    found through the legacy tag `sha256-<hex>.sig` or through OCI referrers. The signature is in the
    `dev.cosignproject.cosign/signature` annotation of the layer, and the payload names the signed digest in
    `critical.image.docker-manifest-digest`;
  - a Sigstore bundle v0.3 (`application/vnd.dev.sigstore.bundle.v0.3+json`), carrying either a DSSE envelope
    or a message signature. A message signature is checked against the simple-signing payload of the same
    manifest, when that payload is stored.
- **Keys**: ECDSA P-256 and P-384, Ed25519, and RSA of 2048 to 8192 bits, with PKCS#1 v1.5 or PSS padding.
  Other keys (P-521, DSA, RSA of another size) are refused when they are added.
- **Not supported**: keyless signatures, which carry a Fulcio certificate, and transparency-log entries
  (Rekor). The `docker-reference` claim is not checked, because proxies and mirrors rename registries.
- **Notation** signatures are listed as present, but they are not verified. They count as `unsupported`.
- A digest that is not `sha256` cannot be verified and is refused as `unsupported`.

## Trusted keys

Trusted keys are kept by the server under a name. A policy chooses its keys by name, and one key may serve
any number of policies. A policy names at most 64 keys.

| Method and path | Use |
|---|---|
| `GET /service/rest/v1/security/keys` | The keys: name, algorithm, public key, fingerprint, notes |
| `POST /service/rest/v1/security/keys` | Add a key. Fields: `name`, `publicKey` (the PEM text) and optional `notes`. Answers `201` |
| `DELETE /service/rest/v1/security/keys/{name}` | Delete a key |

The rights are `astrum:policies:read`, `astrum:policies:create` and `astrum:policies:delete`.

A key is checked strictly when it is added:

- the text is one PEM block labelled `PUBLIC KEY`, a SubjectPublicKeyInfo, at most 16 KiB, with no other data
  around it;
- the DER encoding is canonical: re-encoding the key gives the same bytes;
- elliptic curves are named curves only, and the point is on the curve;
- an Ed25519 key that is a point of small order is refused;
- the RSA public exponent must be at most 2^33 - 1.

The `algorithm` of a key is `ecdsa-p256`, `ecdsa-p384`, `ed25519` or `rsa`. The `fingerprint` is the
SHA-256 of the DER encoding, in hex.

## The policy block `signatures`

```json
"signatures": { "mode": "block", "keys": ["release-2026"] }
```

| Field | Values | Default | Meaning |
|---|---|---|---|
| `mode` | `off`, `report`, `block` | `off` | `off` checks nothing. `report` serves the image and records the result. `block` refuses an image that is not signed |
| `keys` | names of trusted keys | empty | The keys whose signatures count |

A policy in `block` mode whose keys are empty or unusable refuses every manifest, with the reason "the policy
names no usable trusted key".

## How a manifest is decided

A manifest read (a pull by tag or by digest) is checked as follows.

1. A manifest that is exempt (see [what is exempt](#what-is-exempt-and-what-is-not-gated)) is served.
2. The digest of the manifest requested must be signed. It is signed when a valid signature by a key of the
   policy targets it, or targets an index of the same repository that lists it as a child. A signature on an
   index covers its platform images only while that index exists. Up to 20 indexes above the image are
   followed.
3. Only image signatures count. Attestations never do.
4. The signatures are looked up in the repository that holds the manifest. For a group, the first member that
   holds it is used, and that member's policy applies; a policy on the group itself is ignored.

The signature check runs first and the vulnerability gate second: an image that fails the signature check is
refused before its vulnerabilities are looked at (see [Image scanning and the pull gate](image-scanning.md#the-pull-gate)).

### Verifying the candidates

- The legacy tag `sha256-<hex>.sig` is read first. Then the signature manifests of the digest, the ones the
  server marks as signatures, newest first. At most 500 are listed, and at most 200 are read for each digest.
- A signature is verified only after cheap checks: a simple-signing layer needs its signature annotation and a
  stored payload of at most 1 MiB that names the digest; a bundle must be at most 1 MiB. At most 200 candidates
  are verified for each digest.
- On a proxy, when no local candidate signs the digest, the upstream registry is asked for signatures. The
  legacy tag and up to 20 upstream signature manifests are considered, chosen by their artifact type (cosign,
  Sigstore, Notary or signature types, or none). Each verification may fetch at most 60 blobs from upstream. One
  verification runs at a time for each repository and digest.
- A result that stopped at a limit is marked as truncated in the stored detail.

### The result cache and how fast changes apply

Results are kept per repository, in the table `oci_signature_check`. Two kinds of row are kept:

- **decision rows**, keyed by the manifest digest. A row is reused while its fingerprint matches. The
  fingerprint covers the digests of the signature candidates (referrers and the legacy tag), whether the list
  was truncated, and the sorted fingerprints of the policy's keys. A new signature or a change to the key list
  therefore makes the row stale. Tag moves do not affect the rows, because they are keyed by digest. Key names
  are not part of the fingerprint.
- **content rows**, keyed `content:<digest>`, which remember whether a signature artifact is exempt (see below).

Changes to the policy and the keys take effect within about one second on each node. On a proxy:

- a negative decision (not signed) and a negative content result (not exempt) are checked again after
  5 minutes, so a signature published later is seen. A positive result is kept;
- a check that ended in an error is repeated on the next request in `block` mode, and reused for 30 seconds in
  `report` mode.

## What is exempt and what is not gated

A manifest is exempt only when all of the following hold. The server derives its artifact kind from the
manifest content: its artifact type, its config media type and its layer media types. A subject or an artifact
type alone exempts nothing.

1. Its artifact kind is `signature`, `attestation` or `sbom`.
2. It is not an index: it has no `manifests` key, and it has a `layers` array of at most 20 layers.
3. Every layer has a media type from the allow-list: simple signing, a Sigstore bundle, in-toto, DSSE, SPDX JSON,
   CycloneDX JSON, Syft JSON, Notary or JWS and COSE signatures (`application/vnd.cncf.notary.signature`,
   `application/jose+json`, `application/cose`), and the empty OCI descriptor. SPDX or CycloneDX in XML or
   tag-value form are not exempt.
4. Each layer has a `mediaType`, `digest` and `size` key, spelled exactly so, once each. Keys that differ only in
   letter case (in the style of Go decoders) or are repeated make the manifest not exempt. A layer with a `urls`
   key is not exempt.
5. Each layer blob is available and at most 1 MiB, and its length equals the `size` of the descriptor. On a
   hosted repository the blob must be stored. On a proxy a missing blob is fetched from upstream.
6. The blob is one JSON document of the shape its media type promises. The empty descriptor must be exactly
   `{}`. A blob with control bytes (other than tab, line feed and carriage return), or with a 512-byte block that
   looks like a tar header, is not exempt, whatever its shape.

A manifest that fails these checks is judged like any other image. A manifest whose layers could not be read
for a reason other than a rejection (on a hosted repository, a missing blob, or a read error) is also judged
like an image, and that result is not remembered.

Not gated:

- **blobs** and blobs that a push mounts from another repository. Only manifest reads are checked.
- **groups**: the policy on a group is ignored. The policy of the member that holds the manifest applies.

## Exceptions

A signature exception lets an unsigned image through. It needs a policy: `rule` is `signature`, `policy` names
the policy, and it names an image by `digest` or by `name`. A `name` exception may also set `version` to a tag.
Create it with `POST /service/rest/v1/security/exceptions` and the right `astrum:policyexceptions:create`.

- A digest exception covers that manifest.
- An index exception covers the images the index lists, as long as the index exists.
- A name exception covers every pull of that image. With `version` set, it covers only that tag.

## Errors

Errors use the Docker error format, with the code `DENIED` or `UNAVAILABLE`.

| Status | Code | When |
|---|---|---|
| `403` | `DENIED` | The image is unsigned, its signature is invalid or unsupported, or its digest is not `sha256`. The message is `signature: sha256:<12 hex characters> is not signed by a trusted key of policy <name> (<reason>)` |
| `503` | `UNAVAILABLE` | The check ended in an error: a signature or its payload could not be read or fetched, or the check could not finish, for example because the policies could not be read. The message asks for a retry. A stored policy that cannot be parsed refuses the pull with `403` and the reason "the policy cannot be read" |

In `report` mode the image is served in every case, and the result is recorded.

## The listing

`GET /service/rest/v1/docker/{repository}/supply-chain` describes an image. It needs the right to browse the
repository. The parameters are `image` (the image name) and `reference` (a tag or a digest). The answer holds:

- `digest`: the digest of the reference;
- `children`: the child digests of an index;
- `signatures`: each with `digest`, `scheme`, `state` and, when verified, `signer`;
- `attestations` and `sboms`: the digests found (see [SBOMs](sbom.md)).

The `scheme` of a signature is `legacy` (the `.sig` tag), `referrer` (an OCI referrer in the cosign or
simple-signing form), `bundle` (a Sigstore bundle) or `notation`. The `state` of a signature is:

- `verified`: a key of the policy signed it, and `signer` is the name of that key;
- `invalid`: the signature was examined and does not verify;
- `unknown`: the policy names no keys, or the signature or its payload could not be read or fetched;
- `unsupported`: the signature is of a kind the server does not verify, such as Notation.

A signature's `state` is not the decision for the image: the gate decides whether the digest is signed, as
described above.

## Limits

- A pusher who can write to the repository can make a genuine signature unusable: by adding more than 200 newer
  junk signatures, the genuine one is not read. The image is then refused, never served. The check fails
  closed.
- Blobs and cross-repository mounts are not gated.
- A policy on a group is ignored; the policy of the member that holds the manifest applies.
