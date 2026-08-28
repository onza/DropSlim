class Dropslim < Formula
  desc "Optimize images from the command line"
  homepage "https://github.com/onza/DropSlim"
  url "https://github.com/onza/DropSlim/releases/download/v1.6.3/dropslim-cli_1.6.3_aarch64.tar.gz"
  sha256 "f20f75f4839afae639b4a0bd035005981be3279503481456381c3d065cd01c07"
  license "MIT"
  version "1.6.3"

  depends_on arch: :arm64
  depends_on macos: :big_sur

  def install
    libexec.install "dropslim", "vendor", "LICENSE.md", "README.md"
    bin.install_symlink libexec/"dropslim"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/dropslim --version")
  end

  livecheck do
    url :stable
    strategy :github_latest
    regex(/^v?(\d+(?:\.\d+)+)$/i)
  end
end
