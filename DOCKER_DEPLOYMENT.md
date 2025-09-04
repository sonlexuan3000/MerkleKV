# Docker Deployment & Publishing

This document explains how to set up automated Docker publishing to Docker Hub for MerkleKV.

## 🚀 Automated Publishing Setup

### Prerequisites

1. **Docker Hub Account**: Create an account at [hub.docker.com](https://hub.docker.com)
2. **Docker Hub Repository**: Create a repository named `merklekv` (or your preferred name)
3. **GitHub Secrets**: Configure the following secrets in your GitHub repository

### GitHub Secrets Configuration

Go to your GitHub repository → Settings → Secrets and variables → Actions, and add:

#### Required Secrets:
- `DOCKER_USERNAME`: Your Docker Hub username
- `DOCKER_TOKEN`: Your Docker Hub access token (not password!)

#### How to get Docker Hub Token:
1. Go to [hub.docker.com](https://hub.docker.com)
2. Click your profile → Account Settings
3. Go to Security → New Access Token
4. Create a token with "Read, Write, Delete" permissions
5. Copy the token and add it as `DOCKER_TOKEN` secret

### Workflow Triggers

The Docker publishing workflow (`docker-publish.yml`) triggers on:

- ✅ **Version tags**: `v1.0.0`, `v1.2.3`, `v2.0.0-beta.1`, etc.
- ✅ **Manual trigger**: Via GitHub Actions UI
- ❌ **NOT on pushes to main**: Only releases get published

### Image Naming Convention

Images are published as:
- `merklevkv/merklekv:latest` (for latest release)
- `merklevkv/merklekv:v1.0.0` (for specific version)
- `merklevkv/merklekv:v1.0` (for major.minor)
- `merklevkv/merklekv:v1` (for major version)

**Note**: Update the `IMAGE_NAME` in the workflow file to match your Docker Hub username/organization.

## 🏷️ Creating Releases

### Method 1: GitHub CLI
```bash
# Create and push a tag
git tag v1.0.0
git push origin v1.0.0

# Create release with GitHub CLI
gh release create v1.0.0 --title "MerkleKV v1.0.0" --notes "Initial release"
```

### Method 2: GitHub Web UI
1. Go to your repository → Releases
2. Click "Create a new release"
3. Choose a tag (e.g., `v1.0.0`)
4. Add release title and description
5. Click "Publish release"

### Method 3: Git Commands
```bash
# Create and push tag
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

## 🔧 Workflow Features

### Multi-Platform Builds
- **linux/amd64**: Intel/AMD 64-bit processors
- **linux/arm64**: ARM 64-bit processors (Apple Silicon, ARM servers)

### Security Features
- **Vulnerability Scanning**: Trivy scans for security issues
- **SBOM Generation**: Software Bill of Materials for supply chain security
- **Non-root User**: Container runs as non-privileged user

### Performance Optimizations
- **Multi-stage Build**: Minimal production image
- **Layer Caching**: Faster subsequent builds
- **Static Linking**: Self-contained binary

## 📋 Usage After Publishing

Once published, users can pull and run:

```bash
# Pull the latest version
docker pull merklevkv/merklekv:latest

# Run with custom config
docker run -d --name merklekv \
  -p 7379:7379 \
  -v $(pwd)/config.toml:/app/config/config.toml:ro \
  merklevkv/merklekv:latest

# Or use docker-compose
docker-compose up -d
```

## 🧪 Testing the Workflow

### Manual Testing
1. Go to Actions tab in your GitHub repository
2. Select "Docker Build and Publish" workflow
3. Click "Run workflow"
4. Choose a branch and click "Run workflow"

### Local Testing
```bash
# Test the Dockerfile locally
docker build -t merklekv:test .

# Test the image
docker run --rm -p 7379:7379 merklekv:test
```

## 🔍 Troubleshooting

### Common Issues

1. **Authentication Failed**
   - Check `DOCKER_USERNAME` and `DOCKER_TOKEN` secrets
   - Ensure token has correct permissions

2. **Image Name Conflicts**
   - Update `IMAGE_NAME` in workflow to match your Docker Hub username
   - Ensure repository exists on Docker Hub

3. **Build Failures**
   - Check Dockerfile syntax
   - Verify all dependencies are available
   - Review build logs in GitHub Actions

### Workflow Status
- Check the Actions tab for build status
- Review logs for detailed error information
- Security scan results appear in Security tab

## 📚 Additional Resources

- [Docker Hub Documentation](https://docs.docker.com/docker-hub/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Docker Multi-stage Builds](https://docs.docker.com/develop/dev-best-practices/dockerfile_best-practices/)
