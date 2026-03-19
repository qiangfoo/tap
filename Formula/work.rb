class Work < Formula
  desc "Interactive git worktree manager"
  homepage "https://github.com/qiangfoo/homebrew-tap"
  version "0.2.3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/qiangfoo/homebrew-tap/releases/download/v#{version}/work-aarch64-apple-darwin.tar.gz"
      sha256 "8df6043330726a42630bb4ddc1acd56868109f86e7b0e2d23b22283e4c6b351d"
    end
    on_intel do
      url "https://github.com/qiangfoo/homebrew-tap/releases/download/v#{version}/work-x86_64-apple-darwin.tar.gz"
      sha256 "d47e2f6b4944f642c16e6b26e4ab8e4f510afa20f0b78b1ece7ee45b78cb147a"
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
