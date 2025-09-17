# `.github`

<!--

.github/README.md overrides the repository README. CONTRIBUTING also has automatic behavior.

See:

https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes#about-readmes
https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/setting-guidelines-for-repository-contributors
https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/creating-a-default-community-health-file

-->

This directory contains:

* [`actions`](actions/): reusable components for GitHub workflows, with reusable functionality
* `workflows`: the actual workflows that GitHub Actions executes for pushes, pull requests, and others
* [`dependabot.yml`](dependabot.yml): Dependabot configuration
* [CODEOWNERS](CODEOWNERS): automatic assignment of reviewers to PRs.

## Workflows

| Workflow     | Pull requests        | `main` | On-demand[^1] | Scheduled |
| ------------ | -------------------- | ------ | ------------- | --------- |
| `coverage`   | X                    | X      |               |           |
| `validate`   | X                    | X      |               |           |
| `deploy`     | For preview purposes | X      |               |           |
| `release`    |                      |        | X             |           |
| `dependabot` | Some                 |        |               |           |

[^1]: Developers can run most workflows on-demand for development purposes.
      This column lists workflows meant to be run on-demand.

### [`coverage.yaml`](workflows/coverage.yaml)

Runs tests collecting coverage and pushes results to [Codecov](https://app.codecov.io/github/veecle/veecle-os).

### [`dependabot.yaml`](workflows/dependabot.yaml)

Adds one approval to Dependabot PRs, so they require only a single approval.

### [`deploy.yaml`](workflows/deploy.yaml)

Builds and deploys documentation and other websites.

### [`release.yaml`](workflows/release.yaml)

Creates releases and deploys them to registries.

### [`validate.yaml`](workflows/validate.yaml)

Performs all automatic validation, preventing the merge of pull requests that do not pass automatic validation.

## Working with GitHub Actions

You can add the `workflow_dispatch` trigger to a workflow so that GitHub shows a button for executing the workflow on any branch without creating a pull request.
However, GitHub only shows the button for workflows that have the `workflow_dispatch` trigger in the main branch of the repository.

For development, you can enable the `push` trigger in a branch so that GitHub runs the workflow each time you push to the branch.

Adding the `push` trigger temporarily is more convenient for developing workflows, while adding the `workflow_dispatch` trigger is more convenient for workflows that developers might find convenient to run on demand.
