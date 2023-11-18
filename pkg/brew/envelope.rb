class Envelope < Formula
  version "0.3.2"
  desc "A modern environment variables manager"
  homepage "https://github.com/mattrighetti/envelope"
  license :public_domain
  head "https://github.com/mattrighetti/envelope.git", branch: "master"

  if OS.mac?
      url "https://github.com/mattrighetti/envelope/releases/download/#{version}/envelope-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "00b52ad94b678c861b5fb61d43488f13e09d49a4840a94ffd9e519dcc5bebebd"
  elsif OS.linux?
      url "https://github.com/mattrighetti/envelope/releases/download/#{version}/envelope-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "100096b1f710133bda7efd68bf1ad51181bf3ab303a19e7140c885c07b49ea20"
  end

  depends_on "pandoc" => :build

  def install
    bin.install "envelope"

    args = %w[
      --standalone
      --to=man
    ]
    system "pandoc", *args, "man/envelope.1.md", "-o", "envelope.1"
    man1.install "envelope.1"
  end
end
