class Work < Formula
  desc "Interactive git worktree manager"
  homepage "https://github.com/qiangfoo/homebrew-tap"
  version "0.2.3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/qiangfoo/homebrew-tap/releases/download/v#{version}/work-aarch64-apple-darwin.tar.gz"
      sha256 "d314d02ced7622a234466ec8e44e039bec0e63da7ea1110cce3e875044a7e382"
    end
    on_intel do
      url "https://github.com/qiangfoo/homebrew-tap/releases/download/v#{version}/work-x86_64-apple-darwin.tar.gz"
      sha256 "8a8fdb92d57342657bf4f4ad1b676118ed29980d37e3c2b907ab6b8257686a7d"
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
