# MSIX & Microsoft Store Submission Checklist

This document tracks the remaining steps to get FountTUI officially published on the Microsoft Store.

## 1. Preparation
- [ ] **Register Developer Account**: Sign up at the [Microsoft Partner Center](https://partner.microsoft.com/dashboard/registration).
- [ ] **Reserve App Name**: Reserve "Fount" or "FountTUI" in the dashboard.
- [ ] **Collect Identity Info**:
  - Get `Package/Identity/Name` from Partner Center.
  - Get `Package/Identity/Publisher` (starts with `CN=`) from Partner Center.

## 2. Configuration
- [ ] **Update Manifest**: Replace placeholders in `packaging/msix/AppxManifest.xml` with your actual Store IDs.
- [ ] **GitHub Rename**: Ensure the GitHub repository is renamed to `FountTUI` to match URLs in the code.

## 3. Build & Release
- [ ] **Push Version Tag**: Create and push a new tag (e.g., `git tag v0.9.3` then `git push --tags`).
- [ ] **Download MSIX**: Retrieve the `.msix` file from the generated GitHub Release.

## 4. Store Submission
- [ ] **Upload Package**: Upload the `.msix` to the Partner Center.
- [ ] **Store Listing**:
  - [ ] Write description.
  - [ ] Add screenshots (Terminal/TUI view).
  - [ ] Set pricing and availability (Free).
- [ ] **Submit for Review**: Send the submission to Microsoft for approval.

## 5. Post-Approval
- [ ] **Update Wiki**: Add the official Store badge and link back to `Installation.md` and `Home.md`.
- [ ] **Update README**: Add the Store link to the main `README.md`.
