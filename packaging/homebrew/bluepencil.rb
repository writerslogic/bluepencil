class Bluepencil < Formula
  desc "Prose analysis for writers"
  homepage "https://github.com/writerslogic/bluepencil"
  url "https://github.com/writerslogic/bluepencil/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_RELEASE_TARBALL_SHA256"
  license any_of: ["MIT", "Apache-2.0"]
  head "https://github.com/writerslogic/bluepencil.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/bluepencil")
    generate_completions_from_executable(bin/"bluepencil", "completions")
  end

  test do
    (testpath/"t.md").write("# Title\n\nShe ran. She ran again. She ran once more.\n")
    assert_match "3", shell_output("#{bin}/bluepencil count t.md")
    assert_match "repeated-starter", shell_output("#{bin}/bluepencil starters t.md")
  end
end
