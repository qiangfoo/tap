class Work < Formula
  desc "Interactive git worktree manager"
  homepage "https://github.com/qiangfoo/homebrew-work"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/qiangfoo/homebrew-work/releases/download/v#{version}/work-aarch64-apple-darwin.tar.gz"
      sha256 "d7ac27fa5b36054c4f30c2d55cda7ec1828c1381de93e1c1a7c1d9263c7934e0"
    end
    on_intel do
      url "https://github.com/qiangfoo/homebrew-work/releases/download/v#{version}/work-x86_64-apple-darwin.tar.gz"
      sha256 "654bde1badb01099b39527ea9d8b1d6044eaadfdf6916ad0353b8c6e10475cca"
    end
  end

  def install
    bin.install "work"
  end

  def caveats
    <<~EOS
      Add the following to your ~/.zshrc:
        eval "$(work init)"
    EOS
  end
end
