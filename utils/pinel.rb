class Pinel < Formula
  desc "Fast and extensible code editor in rust"
  homepage "https://github.com/you/yourpkg"
  url "https://github.com/you/yourpkg/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "abc123..."  # paste your SHA here
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end

  test do
    system "#{bin}/yourpkg", "--version"
  end
end