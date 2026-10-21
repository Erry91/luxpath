# Security and safety reports

luxpath reads untrusted files (photometric files, scenes, IFC models) and computes values used in life-safety
design. Two kinds of report matter most:

1. **A wrong result that could make a design look safe when it is not**, for example illuminance computed too
   high, a requirement reported as met that is not, or missing data treated as present. These are the most serious
   bugs this project can have (AGENTS.md §2).
2. **A crash, hang or excessive memory use on some input file.** Every input must give a result or an error
   (AGENTS.md §3).

## How to report

- For anything that could be exploited (a crafted file that runs code, reads files, or exhausts a machine), use
  GitHub's **private vulnerability reporting** (Security tab, "Report a vulnerability"). Please do not open a
  public issue for these.
- For wrong results and crashes on ordinary files, open an issue with the **wrong result** or **bug** template.
  Do not attach manufacturers' photometric files or real building plans unless you are free to share them;
  a description or a synthetic file that reproduces the problem is enough.

We aim to acknowledge reports within a week. There is no bug bounty.

## Supported versions

luxpath is pre-alpha: only the latest `main` is supported, and nothing is yet released.
