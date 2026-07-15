---
name: Bug report
about: Something bloomctl does wrong
labels: bug
---

<!--
TENANT DATA CHECK — bloomctl talks to live MDM tenants. Before
submitting, confirm this report contains NO: tenant subdomain or
*.api.kandji.io / *.connect.iru.com hostnames, device serials/UDIDs/
names, user emails, audit-trail lines, or secrets-endpoint output
(FileVault keys, bypass codes, PINs). Sanitization guide:
CONTRIBUTING.md "Sharing logs, payloads, and repros".
-->

**What happened**

**What I expected**

**Repro** (prefer wiremock/fixtures over live-tenant output; run live
repros with `BLOOMCTL_AUDIT=off` if sharing a transcript)

```console
$ bloomctl ...
```

**Environment**
- bloomctl version (`bloomctl --version`):
- OS:
- Install path (brew / mise / source):

- [ ] I confirm this report contains no tenant-identifying data
      (subdomain, hostnames, serials, UDIDs, emails, payloads,
      audit lines, secrets).
