# Git Worktree: Step by Step Explanation

Git worktrees allow you to check out multiple branches of a repository simultaneously in different directories, enabling you to work on several features or fixes concurrently without having to stash or commit incomplete changes.

## How Git Worktree Works

### 1. Repository Structure

When you create a worktree, Git:

- Maintains a single `.git` directory in the main repository
- Creates a lightweight linked working tree in a new location
- Places a special file (`.git`) in the new worktree that points back to the main repository

### 2. Command Execution Process

When you run `git worktree add`:

1. Git verifies the target directory doesn't exist or is empty
2. Creates the target directory if needed
3. Creates a `.git` file (not directory) in the new location that points to the main repository's `.git` directory
4. Registers the new worktree in the main repository's `.git/worktrees/<name>` directory
5. Performs the equivalent of a `git checkout` of the specified branch/commit in the new directory

### 3. Internal Tracking

Git maintains worktree information in:
- `.git/worktrees/` directory in the main repository
- `.git/config` to track worktree details
- `.git/HEAD` in each worktree to track which branch is checked out

## Common Worktree Commands

### Creating a Worktree

```bash
git worktree add ../path/to/directory branch-name
```

This creates a new worktree at the specified path and checks out the specified branch.

### Listing Worktrees

```bash
git worktree list
```

Shows all worktrees associated with the repository, including their paths and branches.

### Removing a Worktree

```bash
git worktree remove path/to/worktree
```

Removes the worktree from Git's tracking. You may need to manually delete the directory.

### Pruning Stale Worktrees

```bash
git worktree prune
```

Removes references to worktrees whose directories no longer exist.

## Local Worktree Synchronization for Agentic Tools

When using worktrees with agentic tools that operate locally, you need strategies to synchronize changes across multiple worktrees without relying on remote repositories. Here are several approaches:

### 1. Purely Local Branch Management

You can maintain a completely local workflow using Git's branch management:

```bash
# Create multiple worktrees for different agents/tasks
git worktree add ../agent1-tree agent1-branch
git worktree add ../agent2-tree agent2-branch
git worktree add ../agent3-tree agent3-branch

# Each agent works in its own tree independently
# When ready to consolidate:

# Option A: Merge all branches into a consolidation branch
git checkout -b consolidation-branch
git merge agent1-branch agent2-branch agent3-branch

# Option B: Cherry-pick specific commits from each branch
git checkout main
git cherry-pick <commit-from-agent1>
git cherry-pick <commit-from-agent2>
git cherry-pick <commit-from-agent3>
```

### 2. Using Patches for Selective Integration

Create and apply patches to transfer changes between worktrees:

```bash
# From agent1's worktree, create a patch
cd ../agent1-tree
git format-patch main -o ../patches/agent1

# From agent2's worktree, apply agent1's changes
cd ../agent2-tree
git am ../patches/agent1/*.patch

# Create a combined solution patch
git format-patch main -o ../patches/combined
```

### 3. Intermediate Integration Branch Strategy

Create an integration branch that different agent worktrees can use to share progress:

```bash
# Create an integration branch
git branch integration main

# Create worktrees with the integration branch as base
git worktree add ../agent1-tree agent1-branch
git worktree add ../agent2-tree agent2-branch

# Periodically, merge changes to the integration branch
cd ../agent1-tree
git checkout agent1-branch
git commit -am "Agent 1 progress"
git checkout integration
git merge agent1-branch

# Other agents can then update from integration
cd ../agent2-tree
git checkout agent2-branch
git merge integration
```

### 4. Automated Local Synchronization Script

Create a script to automate the synchronization process:

```bash
#!/bin/bash
# sync-worktrees.sh

# Commit changes in each worktree
for tree in agent1-tree agent2-tree agent3-tree; do
  cd "../$tree"
  git add .
  git commit -m "Auto-commit from $tree"
done

# Create integration branch if it doesn't exist
cd ../main-repo
git branch integration 2>/dev/null || echo "Integration branch exists"

# Merge all agent branches to integration
git checkout integration
git merge agent1-branch agent2-branch agent3-branch

# Update all worktrees with latest integration
for tree in agent1-tree agent2-tree agent3-tree; do
  cd "../$tree"
  git merge integration
done
```

### 5. File-Level Synchronization

For fine-grained control, synchronize specific files between worktrees:

```bash
# Copy specific files from agent1 to agent2
cp ../agent1-tree/src/specific-file.py ../agent2-tree/src/

# In agent2's worktree, add and commit
cd ../agent2-tree
git add src/specific-file.py
git commit -m "Integrate file from agent1"
```

### 6. Content-Based Merging

When multiple agents modify the same files, merge by content:

```bash
# Create a temporary merging directory
mkdir ../merge-temp
cp ../agent1-tree/shared-file.py ../merge-temp/
cp ../agent2-tree/shared-file.py ../merge-temp/shared-file-agent2.py

# Use merge tools to combine content
cd ../merge-temp
# Use a tool like meld, kdiff3, or vimdiff
meld shared-file.py shared-file-agent2.py

# Copy the merged result back
cp shared-file.py ../agent1-tree/
cp shared-file.py ../agent2-tree/

# Commit in both worktrees
cd ../agent1-tree
git add shared-file.py
git commit -m "Merged content with agent2"

cd ../agent2-tree
git add shared-file.py
git commit -m "Merged content with agent1"
```

### 7. Distributed Agent Workflow Example

For a complex project with multiple agentic tasks:

1. **Setup Phase:**
   ```bash
   # Create worktrees for different concerns
   git worktree add ../research-agent research-branch
   git worktree add ../coding-agent coding-branch
   git worktree add ../testing-agent testing-branch
   git worktree add ../integration integration-branch
   ```

2. **Working Phase:**
   - Research agent explores and drafts in its worktree
   - Coding agent implements in its worktree
   - Testing agent writes tests in its worktree

3. **Synchronization Points:**
   ```bash
   # Research shares findings with coding
   cd ../research-agent
   git format-patch main -o ../patches/research-findings
   
   cd ../coding-agent
   git am ../patches/research-findings/*.patch
   
   # Coding shares implementation with testing
   cd ../coding-agent
   git format-patch main -o ../patches/implementation
   
   cd ../testing-agent
   git am ../patches/implementation/*.patch
   
   # All changes get merged to integration
   cd ../integration
   git merge research-branch coding-branch testing-branch
   ```

4. **Final Consolidation:**
   ```bash
   # Verify the integrated solution works
   cd ../integration
   # Run tests, verify functionality
   
   # Merge back to main
   cd /path/to/main-repo
   git checkout main
   git merge integration
   ```

## Benefits of Local Worktree Synchronization

- **Parallelization:** Multiple agents can work simultaneously on different aspects
- **Isolation:** Each agent's work remains isolated until explicitly shared
- **Traceability:** Each agent's contributions remain distinct in the commit history
- **Flexibility:** Choose the right integration strategy based on project needs
- **No Network Dependency:** Works entirely offline, no remote repository needed

## Merging from a Worktree Back to Main

When you've completed work in a worktree branch and need to merge it back to the main branch, you have several options. Here's a step-by-step guide:

### Option 1: Merge from Within the Main Working Directory

```bash
# From the main directory (with main branch checked out)
git fetch --all                           # Update all branch information
git merge hotfix-branch                   # Merge the branch from the worktree
```

### Option 2: Create a Pull Request (for GitHub/GitLab workflows)

```bash
# From the worktree directory
git push origin hotfix-branch             # Push the branch to remote
# Then create a PR through the GitHub/GitLab interface
```

### Option 3: Rebase the Worktree Branch Before Merging

If the main branch has progressed while you were working in the worktree, you may want to rebase your changes to avoid merge conflicts:

```bash
# First, update your main branch in the primary working directory
cd /path/to/main-repo
git checkout main
git pull                                  # Get latest main changes

# Then go to your worktree and rebase
cd /path/to/worktree
git fetch                                 # Update branch information
git rebase origin/main                    # Rebase onto the latest main

# Resolve any conflicts during rebase
# For each conflict:
# 1. Edit files to fix conflicts
# 2. git add <resolved-files>
# 3. git rebase --continue

# Once rebase is complete, push the rebased branch (may need force)
git push --force-with-lease origin hotfix-branch

# Finally, merge from the main directory
cd /path/to/main-repo
git checkout main
git merge hotfix-branch                   # Should now be a clean fast-forward merge
```

### Option 4: Cherry-Pick Specific Commits

If you only want to apply specific changes:

```bash
# From the main directory (with main branch checked out)
git cherry-pick <commit-hash>             # Apply a specific commit from the worktree branch
```

### Avoiding Merge Conflicts When Main is Ahead

To minimize merge conflicts when the main branch has progressed:

1. **Regularly update your branch:**
   ```bash
   # In worktree directory
   git fetch origin
   git merge origin/main                  # Or git rebase origin/main
   ```

2. **Keep changes focused and atomic:**
   - Make smaller, targeted changes in worktree branches
   - Avoid massive refactorings that touch many files

3. **Use rebase instead of merge:**
   ```bash
   git rebase origin/main                 # Replay your changes on top of latest main
   ```

4. **Consider feature flags:**
   - For long-lived branches, consider using feature flags
   - Allows partial integration while keeping features disabled

5. **Communicate with your team:**
   - Coordinate major changes to overlapping areas
   - Use pull request descriptions to explain changes that might conflict

6. **Resolve conflicts interactively during rebase:**
   ```bash
   git rebase -i origin/main              # Interactive rebase for more control
   ```

After successfully merging and verifying the changes, you can clean up:

```bash
git branch -d hotfix-branch               # Delete the branch if no longer needed
git worktree remove /path/to/worktree     # Remove the worktree
```

## Benefits of Worktrees

- Eliminate the need to stash changes when switching contexts
- Work on multiple branches simultaneously
- Test interactions between different branches
- Build and test multiple versions of code at once
- Maintain separate working directories for different tasks

## Limitations

- Some Git operations (like `git submodule`) may need to be performed in the main repository
- Worktrees share the same objects database, so they use the same refs and objects
- Can't check out the same branch in multiple worktrees (would create conflicts)