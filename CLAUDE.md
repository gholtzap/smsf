dont write or edit any .md or markdown files EXCEPT FOR README.md.

never write any comments.

use types. If you find yourself declaring the same exact type twice, create a type for it so we can reuse it.

never push my mongodb credentials to github.

after every successful completion of a task, do the following:
1. Build this project to make sure there are no errors.
2. Commit everything to git with a commit message no longer than 8 words, do not write a commit description. NEVER SAY CO-AUTHORED BY CLAUDE, DO NOT TAG YOURSELF. and push directly to master.

NEVER WRITE ANYTHING LIKE THIS:
🤖 Generated with Claude Code

Co-Authored-By: Claude noreply@anthropic.com
EOF

### PROJECT FILES EXPLANATION

This NF (Network function) is part of a larger project, where I am making an entire 5G Mobile Core. You are allowed and encouraged to explore the other NFs, such as the AUSF, UDM, etc. In telecom, many requests happen between NFs, so it may be useful to see what the source code of another NF is expecting or sending in order to diagnose issues. All my source code is located inside the dev/telco/ repository as follows:

5g-core/ : All NFs to make up my custom 5G core.

Testing & Simulation:
  - UERANSIM: UE and RAN simulator (C++)
  - test-free5gc: Testing environment