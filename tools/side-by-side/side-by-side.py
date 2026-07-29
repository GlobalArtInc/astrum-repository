#!/usr/bin/env python3
import argparse
import os
import shutil
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from sbs import harness
from sbs.client import HarnessError, Server, read_password
from sbs import existing
from sbs import scenarios

HERE = os.path.dirname(os.path.abspath(__file__))


def arguments():
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--reference-url", default=os.environ.get("SBS_REFERENCE_URL", "http://127.0.0.1:8081"))
    common.add_argument("--reference-user", default=os.environ.get("SBS_REFERENCE_USER", "admin"))
    common.add_argument("--reference-password-file", default=os.environ.get("SBS_REFERENCE_PASSWORD_FILE"))
    common.add_argument("--candidate-url", default=os.environ.get("SBS_CANDIDATE_URL"))
    common.add_argument("--candidate-user", default=os.environ.get("SBS_CANDIDATE_USER", "admin"))
    common.add_argument("--candidate-password-file", default=os.environ.get("SBS_CANDIDATE_PASSWORD_FILE"))
    common.add_argument("--prefix", default="m5s-", help="name prefix of every object the harness creates")
    common.add_argument("--allow-list", default=os.path.join(HERE, "allow-list.json"))
    common.add_argument("--report-json", help="write the full result as JSON to this file")
    common.add_argument("--work-dir", help="scratch directory (default: a new temporary directory)")
    common.add_argument("--keep-work", action="store_true", help="keep the scratch directory")
    common.add_argument("--show-allowed", action="store_true", help="print allowed differences too")
    common.add_argument("--strict-json", action="store_true",
                        help="also report JSON keys only the candidate sends (clients ignore unknown keys)")
    common.add_argument("--max-lines", type=int, default=12, help="differences printed per step")
    common.add_argument("--insecure", action="store_true", help="do not verify TLS certificates")
    common.add_argument("--timeout", type=int, default=120)
    common.add_argument("--port-base", type=int, default=int(os.environ.get("SBS_PORT_BASE", "18610")),
                        help="first of 90 local ports for upstream and recording servers")
    common.add_argument("-v", "--verbose", action="store_true")
    parser = argparse.ArgumentParser(description="Compare client behaviour of two Astrum Repository servers.")
    sub = parser.add_subparsers(dest="command", required=True)
    run = sub.add_parser("run", parents=[common], help="create the same objects on both servers and compare scenarios")
    run.add_argument("--scenario", action="append", default=[], help="scenario name or glob (repeatable)")
    run.add_argument("--skip", action="append", default=[], help="scenario name or glob to leave out (repeatable)")
    run.add_argument("--keep", action="store_true", help="keep the repositories and objects created on both servers")
    run.add_argument("--proxies", choices=("auto", "on", "off"), default="auto",
                     help="proxy scenarios need both servers to allow 127.0.0.1 in their SSRF settings")
    run.add_argument("--allow-loopback", action="append", default=[], choices=harness.SIDES,
                     help="add 127.0.0.1 to the SSRF allow list of that server while proxy scenarios run, then restore it")
    ex = sub.add_parser("existing", parents=[common], help="compare what both servers already hold, read only")
    ex.add_argument("--repository", action="append", default=[], help="repository name or glob (repeatable)")
    ex.add_argument("--types", default="hosted,group", help="repository types to read (hosted,group,proxy)")
    ex.add_argument("--max-assets", type=int, default=200, help="assets fetched per repository (0 = all)")
    ex.add_argument("--no-rest", action="store_true", help="skip the configuration and security listings")
    sub.add_parser("list", parents=[common], help="list the scenarios")
    args = parser.parse_args()
    args.scenario = getattr(args, "scenario", [])
    return args


def main():
    args = arguments()
    log = harness.Log(args.verbose)
    if args.command == "list":
        for item in harness.SCENARIOS:
            log.info("%-14s %s" % (item.name, item.description))
        return 0
    if not args.candidate_url:
        raise HarnessError("--candidate-url is required")
    servers = {
        "reference": Server("reference", args.reference_url, args.reference_user,
                            read_password(args.reference_password_file, "reference"), args.timeout, args.insecure),
        "candidate": Server("candidate", args.candidate_url, args.candidate_user,
                            read_password(args.candidate_password_file, "candidate"), args.timeout, args.insecure),
    }
    for side, server in servers.items():
        response = server.request("GET", "/service/rest/v1/status")
        if response.status != 200:
            raise HarnessError("%s at %s answers status %s" % (side, server.base, response.status))
        response = server.request("GET", "/service/rest/v1/repositories", headers={"Accept": "application/json"})
        if response.status != 200:
            raise HarnessError("%s rejects the credentials of %s (HTTP %s)" % (side, server.user, response.status))
    work = args.work_dir or tempfile.mkdtemp(prefix="side-by-side-")
    os.makedirs(work, exist_ok=True)
    from sbs import compare
    compare.SUPERSET[0] = not args.strict_json
    context = harness.Context(args, servers, work, log)
    allow = harness.load_allow_list(args.allow_list)
    try:
        if args.command == "existing":
            outcomes = existing.run(context, allow)
        else:
            outcomes = scenarios.run(context, allow)
        ok = harness.report(context, outcomes, allow, args.report_json)
    finally:
        if not args.keep_work:
            shutil.rmtree(work, ignore_errors=True)
    return 0 if ok else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except HarnessError as error:
        print("side-by-side: %s" % error, file=sys.stderr)
        sys.exit(2)
    except KeyboardInterrupt:
        sys.exit(130)
