class Tide < Formula
  desc "Terminal Integrated Development Environment for parallel AI coding agents"
  homepage "https://github.com/kurarensu16/TIDE"
  version "0.1.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/kurarensu16/TIDE/releases/download/v#{version}/tide-v#{version}-macos-arm64.tar.gz"
      # sha256 "<macos-arm64-sha256>"
    else
      url "https://github.com/kurarensu16/TIDE/releases/download/v#{version}/tide-v#{version}-macos-x86_64.tar.gz"
      # sha256 "<macos-x86_64-sha256>"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/kurarensu16/TIDE/releases/download/v#{version}/tide-v#{version}-linux-x86_64.tar.gz"
      # sha256 "<linux-x86_64-sha256>"
    end
  end

  def install
    bin.install "tide"
  end

  test do
    assert_match "tide", shell_output("#{bin}/tide --help")
  end
end

