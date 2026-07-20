import contextlib
import fnmatch
import json
import os
import re
import secrets
import shutil
import subprocess
import sys
import time
import traceback
import urllib.parse

from . import compare
from .compare import Observation, Spec
from .client import DEFAULT, HarnessError
from .normalize import Normalizer
from .upstream import Recorder, Upstream

SCENARIOS = []
SIDES = ("reference", "candidate")
PATH_FORMATS = {"maven2": "maven"}


class Skip(Exception):
    pass


class Scenario:
    def __init__(self, name, fn, formats, clients, description, proxies):
        self.name = name
        self.fn = fn
        self.formats = formats
        self.clients = clients
        self.description = description
        self.proxies = proxies


def scenario(name, formats=(), clients=(), proxies=False):
    def wrap(fn):
        SCENARIOS.append(Scenario(name, fn, tuple(formats), tuple(clients), (fn.__doc__ or "").strip(), proxies))
        return fn
    return wrap


class Context:
    def __init__(self, args, servers, work, log):
        self.args = args
        self.servers = servers
        self.work = work
        self.log = log
        self.prefix = args.prefix
        self.keep = getattr(args, "keep", False)
        self.state = {}
        self.volatile = ("downloadCount", "lastDownloaded") if args.command == "run" else ()
        self.client_users = {}
        self.free_ports = list(range(args.port_base + 1, args.port_base + 90))
        self._upstream = None
        self.closers = []

    def on_close(self, fn):
        self.closers.append(fn)

    def port(self):
        if not self.free_ports:
            raise HarnessError("no free local port left")
        return self.free_ports.pop(0)

    def release(self, port):
        self.free_ports.append(port)

    def upstream(self):
        if self._upstream is None:
            self._upstream = Upstream(self.args.port_base)
        return self._upstream

    def close(self):
        if self._upstream is not None:
            self._upstream.close()
            self._upstream = None
        for fn in reversed(self.closers):
            try:
                fn()
            except Exception as error:
                self.log.info("   closing failed: %s" % error)
        self.closers = []


class Trial:
    def __init__(self, context, scenario, side):
        self.context = context
        self.scenario = scenario
        self.side = side
        self.server = context.servers[side]
        self.prefix = context.prefix
        self.observations = {}
        self.order = []
        self.bases = [self.server.base] + [s.base for s in context.servers.values() if s.base != self.server.base]
        self.extra_rules = []
        self.cleanups = []
        self.work = os.path.join(context.work, scenario.name, side)
        shutil.rmtree(self.work, ignore_errors=True)
        os.makedirs(self.work, exist_ok=True)
        self.shared = context.state.setdefault(scenario.name, {})
        self.vars = {}
        self.skipped = None
        self.error = None

    @property
    def log(self):
        return self.context.log

    def name(self, suffix):
        return self.prefix + suffix

    def client_user(self):
        return self.context.client_users[self.side]

    def path(self, *parts):
        target = os.path.join(self.work, *parts)
        os.makedirs(os.path.dirname(target), exist_ok=True)
        return target

    def need(self, *commands):
        for command in commands:
            if not shutil.which(command):
                raise Skip("%s is not installed" % command)

    def normalizer(self):
        return Normalizer(self.bases, self.extra_rules, self.context.volatile)

    def normalize_rule(self, pattern, replacement):
        self.extra_rules.append((pattern, replacement))

    def add_base(self, base):
        if base not in self.bases:
            self.bases.append(base)

    def _key(self, step):
        key = step
        n = 2
        while key in self.observations:
            key = "%s#%d" % (step, n)
            n += 1
        self.order.append(key)
        return key

    def request(self, method, path, body=None, headers=None, auth=DEFAULT, server=None, digest=False):
        return (server or self.server).request(method, path, body=body, headers=headers, auth=auth, digest=digest)

    def step(self, step, method, path, body=None, headers=None, auth=DEFAULT, spec=None, server=None, digest=False,
             **spec_args):
        response = self.request(method, path, body=body, headers=headers, auth=auth, server=server, digest=digest)
        self.record_response(step, response, spec or Spec(**spec_args))
        return response

    def record_response(self, step, response, spec):
        parsed = urllib.parse.urlsplit(response.url)
        shown = parsed.path + ("?" + parsed.query if parsed.query else "")
        data = {"method": response.method, "path": shown, "status": response.status,
                "headers": response.headers(), "body": response.body}
        key = self._key(step)
        self.observations[key] = Observation(key, "http", data, spec, self.normalizer())

    def fact(self, step, value, spec=None, note=None, **spec_args):
        key = self._key(step)
        self.observations[key] = Observation(key, "fact", {"value": value}, spec or Spec(**spec_args),
                                             self.normalizer(), note)
        return value

    def run(self, step, argv, env=None, cwd=None, input=None, timeout=600, record=True, check=False):
        full_env = dict(os.environ)
        full_env.update(env or {})
        started = time.monotonic()
        try:
            process = subprocess.run(argv, cwd=cwd or self.work, env=full_env, input=input, capture_output=True,
                                     timeout=timeout)
            code = process.returncode
            output = (process.stdout or b"").decode("utf-8", "replace") + (process.stderr or b"").decode("utf-8", "replace")
        except subprocess.TimeoutExpired as error:
            code = "timeout"
            output = ((error.stdout or b"") + (error.stderr or b"")).decode("utf-8", "replace")
        self.log.debug("%s %s/%s: exit %s in %.1fs" % (self.side, self.scenario.name, step, code, time.monotonic() - started))
        tail = "\n".join(output.strip().splitlines()[-15:])
        if record:
            self.fact(step, code, note=tail)
        if check and code != 0:
            raise HarnessError("%s failed (%s):\n%s" % (" ".join(argv), code, tail))
        return code, output

    @contextlib.contextmanager
    def recording(self, prefix, spec=None, keep=None):
        port = self.context.port()
        recorder = Recorder(port, self.server.base)
        self.add_base(recorder.base)
        try:
            yield recorder.base
        finally:
            recorder.close()
            self.context.release(port)
            normalizer = self.normalizer()
            for exchange in recorder.exchanges:
                shown = normalizer.text(exchange.path)
                if keep is not None and not keep(exchange.method, shown):
                    continue
                data = {"method": exchange.method, "path": exchange.path, "status": exchange.status,
                        "headers": {}, "body": exchange.body}
                for key, value in exchange.headers:
                    key = key.lower()
                    data["headers"][key] = data["headers"][key] + ", " + value if key in data["headers"] else value
                key = self._key("%s %s %s" % (prefix, exchange.method, shown))
                chosen = spec(exchange.method, shown) if callable(spec) else spec
                self.observations[key] = Observation(key, "http", data, chosen or Spec(), normalizer)

    def defer(self, fn, *args):
        self.cleanups.append((fn, args))

    def delete_repository(self, name):
        response = self.server.request("DELETE", "/service/rest/v1/repositories/" + urllib.parse.quote(name))
        if response.status not in (204, 404):
            raise HarnessError("%s: cannot delete repository %s: %s %s" % (self.side, name, response.status,
                                                                          response.text()[:200]))

    def create_repository(self, fmt, kind, body, step=None):
        name = body["name"]
        self.delete_repository(name)
        path = "/service/rest/v1/repositories/%s/%s" % (PATH_FORMATS.get(fmt, fmt), kind)
        response = self.step(step or "create %s-%s" % (fmt, kind), "POST", path, body=json.dumps(body),
                             headers={"Content-Type": "application/json", "Accept": "application/json"},
                             spec=Spec(body="json"))
        if not self.context.keep:
            self.defer(self.delete_repository, name)
        if response.status not in (200, 201, 204):
            raise HarnessError("%s: cannot create %s repository %s: %s %s" % (self.side, fmt, name, response.status,
                                                                             response.text()[:300]))
        self.json_step((step or "create %s-%s" % (fmt, kind)) + " config", "%s/%s" % (path, name))
        return name

    def repo_url(self, name):
        return "%s/repository/%s/" % (self.server.base, name)

    def json_step(self, step, path, spec=None, **spec_args):
        args = {"body": "json"}
        args.update(spec_args)
        return self.step(step, "GET", path, headers={"Accept": "application/json"}, spec=spec or Spec(**args))


def blob_store(name="default"):
    return {"blobStoreName": name, "strictContentTypeValidation": True, "writePolicy": "ALLOW"}


def load_allow_list(path):
    if not path or not os.path.exists(path):
        return []
    with open(path, "r", encoding="utf-8") as handle:
        entries = json.load(handle)
    for entry in entries:
        if not entry.get("reason"):
            raise HarnessError("allow list entry %s has no reason" % entry.get("id"))
        entry["used"] = 0
    return entries


def glob(value, pattern):
    expression = "".join(".*" if c == "*" else re.escape(c) for c in pattern)
    return re.fullmatch(expression, value, re.S) is not None


def match_allow(entries, mode, difference):
    for entry in entries:
        if entry.get("mode") not in (None, mode):
            continue
        if not glob(difference.id, entry["id"]):
            continue
        if entry.get("field") and not glob(difference.field, entry["field"]):
            continue
        if "reference" in entry and not glob(str(difference.reference), entry["reference"]):
            continue
        if "candidate" in entry and not glob(str(difference.candidate), entry["candidate"]):
            continue
        entry["used"] += 1
        return entry
    return None


class Log:
    def __init__(self, verbose):
        self.verbose = verbose

    def info(self, message):
        print(message, flush=True)

    def debug(self, message):
        if self.verbose:
            print("  . " + message, flush=True)


def run_trial(context, item, side):
    trial = Trial(context, item, side)
    try:
        item.fn(trial)
    except Skip as error:
        trial.skipped = str(error)
    except Exception as error:
        trial.error = "".join(traceback.format_exception_only(type(error), error)).strip()
        context.log.debug(traceback.format_exc())
        trial.fact("error", trial.error)
    return trial


def cleanup(context, trials):
    for trial in trials:
        for fn, args in reversed(trial.cleanups):
            try:
                fn(*args)
            except Exception as error:
                context.log.info("  cleanup on %s failed: %s" % (trial.side, error))


def compare_trials(item, trials, allow, mode):
    ref, cand = trials
    results = []
    keys = list(ref.order) + [k for k in cand.order if k not in ref.observations]
    for key in keys:
        step_id = "%s/%s" % (item.name, key)
        differences = compare.compare(step_id, ref.observations.get(key), cand.observations.get(key))
        for difference in differences:
            entry = match_allow(allow, mode, difference)
            difference.allowed = entry["reason"] if entry else None
            difference.status = entry.get("status", "accepted") if entry else None
            if difference.field in ("status", "value", "exists") and not difference.detail:
                notes = [o.note for o in (ref.observations.get(key), cand.observations.get(key)) if o is not None]
                if any(notes):
                    difference.detail = "\n".join("%s: %s" % (s, (o.note if o else "") or "") for s, o in
                                                  zip(SIDES, (ref.observations.get(key), cand.observations.get(key))))
        results.append((key, differences))
    return results


def indent(text, prefix="      "):
    return "\n".join(prefix + line for line in str(text).splitlines())


def report(context, outcomes, allow, json_path):
    log = context.log
    totals = {"scenarios": 0, "skipped": 0, "steps": 0, "equal": 0, "allowed": 0, "different": 0}
    document = {"mode": context.args.command, "reference": context.servers["reference"].base,
                "candidate": context.servers["candidate"].base, "scenarios": []}
    for item, trials, results in outcomes:
        entry = {"name": item.name, "steps": [], "skipped": None}
        if any(t.skipped for t in trials):
            reason = "; ".join("%s: %s" % (t.side, t.skipped) for t in trials if t.skipped)
            entry["skipped"] = reason
            totals["skipped"] += 1
            log.info("-- %s: skipped (%s)" % (item.name, reason))
            document["scenarios"].append(entry)
            continue
        totals["scenarios"] += 1
        equal = sum(1 for _, d in results if not d)
        unexpected = sum(1 for _, d in results if any(x.allowed is None for x in d))
        allowed_only = len(results) - equal - unexpected
        totals["steps"] += len(results)
        totals["equal"] += equal
        totals["allowed"] += allowed_only
        totals["different"] += unexpected
        flag = "OK" if unexpected == 0 else "DIFF"
        log.info("-- %s: %d steps, %d equal, %d allowed, %d different [%s]" % (item.name, len(results), equal,
                                                                          allowed_only, unexpected, flag))
        limit = context.args.max_lines
        for key, differences in results:
            entry["steps"].append({"step": key, "differences": [d.as_json() for d in differences]})
            shown = 0
            for d in differences:
                if d.allowed is not None and not context.args.show_allowed:
                    continue
                shown += 1
                if shown > limit:
                    rest = [x for x in differences if x.allowed is None or context.args.show_allowed]
                    log.info("   ... %d more differences in %s/%s (see --report-json)" % (len(rest) - limit, item.name, key))
                    break
                tag = "allowed" if d.allowed is not None else "DIFF"
                log.info("   %s %s %s: reference %s | candidate %s" % (tag, d.id, d.field, d.reference, d.candidate))
                if d.allowed is not None:
                    log.info(indent("reason: " + d.allowed))
                if d.detail and d.allowed is None:
                    log.info(indent(d.detail))
        document["scenarios"].append(entry)
    opened = [e for e in allow if e["used"] and e.get("status") == "open"]
    if opened:
        log.info("open differences (on the allow list, for the owner to decide):")
        for e in opened:
            log.info("   %s %s: %d steps" % (e["id"], e.get("field", ""), e["used"]))
            log.info(indent(e["reason"]))
    document["open"] = [{"id": e["id"], "field": e.get("field"), "steps": e["used"], "reason": e["reason"]}
                        for e in opened]
    stale = [e for e in allow if not e["used"] and e.get("mode") in (None, context.args.command)]
    if stale and context.args.show_allowed and not context.args.scenario:
        log.info("allow list entries that matched nothing:")
        for e in stale:
            log.info("   %s %s" % (e["id"], e.get("field", "")))
    log.info("== %(scenarios)d scenarios run, %(skipped)d skipped; %(steps)d steps: %(equal)d equal, "
             "%(allowed)d with allowed differences only, %(different)d with unexpected differences" % totals)
    document["totals"] = totals
    if json_path:
        with open(json_path, "w", encoding="utf-8") as handle:
            json.dump(document, handle, indent=2, default=str)
    return totals["different"] == 0


def random_password():
    return secrets.token_urlsafe(18)


def ensure_client_users(context):
    for side, server in context.servers.items():
        password = random_password()
        user = context.prefix + "client"
        server.request("DELETE", "/service/rest/v1/security/users/" + user)
        body = {"userId": user, "firstName": "Side", "lastName": "By Side", "emailAddress": user + "@example.invalid",
                "password": password, "status": "active", "roles": [context.admin_roles[side]]}
        server.json("POST", "/service/rest/v1/security/users", body, ok=(200, 201))
        context.client_users[side] = (user, password)


def drop_client_users(context):
    for side, server in context.servers.items():
        user = context.prefix + "client"
        server.request("DELETE", "/service/rest/v1/security/users/" + user)


def selected(names):
    if not names:
        return list(SCENARIOS)
    out = []
    for item in SCENARIOS:
        if any(fnmatch.fnmatchcase(item.name, pattern) for pattern in names):
            out.append(item)
    if not out:
        raise HarnessError("no scenario matches %s" % ", ".join(names))
    return out


def print_progress(log, item, side, trial):
    status = "skipped: " + trial.skipped if trial.skipped else ("error: " + trial.error if trial.error else
                                                                 "%d steps" % len(trial.order))
    log.info("   %s on %s: %s" % (item.name, side, status))


def run_scenarios(context, items, allow, mode):
    outcomes = []
    for item in items:
        context.log.info(">> %s" % item.name)
        trials = []
        for side in SIDES:
            trial = run_trial(context, item, side)
            print_progress(context.log, item, side, trial)
            trials.append(trial)
            if trial.skipped:
                break
        if len(trials) == 1:
            trials.append(trials[0])
            trials[1] = Trial.__new__(Trial)
            trials[1].__dict__.update(trials[0].__dict__)
            trials[1].side = "candidate"
            trials[1].cleanups = []
        results = compare_trials(item, trials, allow, mode)
        cleanup(context, trials)
        outcomes.append((item, trials, results))
    return outcomes


def exit_code(ok):
    sys.exit(0 if ok else 1)
