import gzip
import hashlib
import io
import tarfile
import zipfile

from ..compare import Spec
from ..harness import scenario
from .common import NONE, group, hosted, multipart, rest_listing, settle_search



def module(top):
    out = io.BytesIO()
    with tarfile.open(fileobj=out, mode="w") as archive:
        data = b'output "greeting" {\n  value = "hello"\n}\n'
        info = tarfile.TarInfo("%s/main.tf" % top)
        info.size = len(data)
        info.mtime = 1577836800
        archive.addfile(info, io.BytesIO(data))
    return gzip.compress(out.getvalue(), mtime=0)


def provider(name):
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w") as archive:
        info = zipfile.ZipInfo(name, (2020, 1, 1, 0, 0, 0))
        info.external_attr = 0o100755 << 16
        archive.writestr(info, "#!/bin/sh\necho %s\n" % name)
    return out.getvalue()


def strip_query(value):
    return value.split("?", 1)[0] if isinstance(value, str) else value


@scenario("terraform", formats=["terraform"])
def terraform(t):
    """terraform hosted and group: module and provider registry protocols, discovery, network mirror, uploads."""
    p = t.prefix
    repo = t.create_repository("terraform", "hosted", hosted(p + "tf-hosted", write_policy="ALLOW_ONCE"),
                               step="create hosted")
    grp = t.create_repository("terraform", "group", group(p + "tf-group", [repo]), step="create group")
    base = "/repository/%s/v1/modules/sbs/vpc/aws/" % repo
    t.step("put module", "PUT", base + "1.2.0/package.tar.gz", body=module("vpc"),
           headers={"Content-Type": "application/gzip"}, spec=NONE)
    t.step("put module zip", "PUT", base + "1.10.0/vpc.zip", body=module("vpc"),
           headers={"Content-Type": "application/zip"}, spec=NONE)
    t.step("put module again", "PUT", base + "1.2.0/x.tar.gz", body=module("vpc"),
           headers={"Content-Type": "application/gzip"}, spec=NONE)
    t.step("put module tgz", "PUT", base + "1.10.0/vpc.tgz", body=module("vpc"),
           headers={"Content-Type": "application/gzip"}, spec=NONE)
    t.step("put not archive", "PUT", base + "2.0.0/vpc.tar.gz", body=b"plain text",
           headers={"Content-Type": "application/gzip"}, spec=NONE)
    t.step("versions", "GET", base + "versions", spec=Spec(body="json"))
    t.step("versions anonymous", "GET", base + "versions", auth=None, spec=NONE)
    t.step("versions missing", "GET", "/repository/%s/v1/modules/sbs/none/aws/versions" % repo, spec=Spec(body="json"))
    download = t.step("download", "GET", base + "1.2.0/download", spec=NONE, extra_headers=("x-terraform-get",),
                      skip_headers=("x-terraform-get",))
    location = download.header("X-Terraform-Get") or ""
    t.fact("download location", strip_query(location))
    if "?" in location:
        t.step("archive", "GET", base + "1.2.0/vpc-aws-1.2.0.tar.gz?" + location.split("?", 1)[1], auth=None,
               spec=Spec(body="bytes"))
    t.step("archive unsigned", "GET", base + "1.2.0/vpc-aws-1.2.0.tar.gz", auth=None, spec=NONE)
    t.step("discovery", "GET", "/repository/%s/.well-known/terraform.json" % repo, auth=None, spec=Spec(body="json"))
    t.step("group versions", "GET", "/repository/%s/v1/modules/sbs/vpc/aws/versions" % grp, spec=Spec(body="json"))
    t.step("group discovery", "GET", "/repository/%s/.well-known/terraform.json" % grp, auth=None, spec=Spec(body="json"))
    t.step("token", "GET", "/repository/%s/token" % repo, spec=NONE)

    archive = "terraform-provider-demo_2.0.0_linux_amd64.zip"
    package = provider("terraform-provider-demo_v2.0.0")
    wrong = ("%s  %s\n" % ("0" * 64, archive)).encode()
    body, ctype = multipart([("terraform.namespace", "sbs"), ("terraform.type", "demo"), ("terraform.version", "2.0.0")],
                            [("terraform.asset1", archive, package, None),
                             ("terraform.asset2", "terraform-provider-demo_2.0.0_SHA256SUMS", wrong, None)])
    t.step("provider upload wrong sums", "POST", "/service/rest/v1/components?repository=" + repo, body=body,
           headers={"Content-Type": ctype}, spec=NONE)
    body, ctype = multipart([("terraform.namespace", "sbs"), ("terraform.type", "demo"), ("terraform.version", "v2.0.0"),
                             ("terraform.protocols", "6")], [("terraform.asset", archive, package, None)])
    t.step("provider upload", "POST", "/service/rest/v1/components?repository=" + repo, body=body,
           headers={"Content-Type": ctype}, spec=NONE)
    pbase = "/repository/%s/v1/providers/sbs/demo/" % repo
    t.step("provider versions", "GET", pbase + "versions", spec=Spec(body="json"))
    t.step("provider sums", "GET", pbase + "2.0.0/terraform-provider-demo_2.0.0_SHA256SUMS", spec=Spec(body="bytes"))
    t.fact("provider sums expected", "%s  %s\n" % (hashlib.sha256(package).hexdigest(), archive))
    t.step("provider download unsigned", "GET", pbase + "2.0.0/download/linux/amd64", spec=NONE)
    t.step("mirror index", "GET", "/repository/%s/registry.terraform.io/sbs/demo/index.json" % repo, spec=Spec(body="json"))
    t.step("mirror version", "GET", "/repository/%s/registry.terraform.io/sbs/demo/2.0.0.json" % repo, spec=NONE)
    rest_listing(t, "rest", repo)
    settle_search(t, repo)
    t.json_step("search modules", "/service/rest/v1/search?repository=%s&terraform.kind=module" % repo)
