// SPDX-License-Identifier: MIT OR Apache-2.0
// Small shared views: the mascot, the primary button and a card.

import SwiftUI

enum Brand {
    static let blue = Color(red: 0.07, green: 0.36, blue: 0.93)   // Khadag Blue
    static let dough = Color(red: 1.0, green: 0.78, blue: 0.30)
}

struct Buddy: View {
    var size: CGFloat = 96
    var body: some View {
        if let url = Bundle.main.url(forResource: "buuz-buddy-256", withExtension: "png"),
            let image = NSImage(contentsOf: url)
        {
            Image(nsImage: image).resizable().interpolation(.high)
                .frame(width: size, height: size)
        } else {
            Text("🥟").font(.system(size: size * 0.7))
        }
    }
}

/// The website's pressed button: a solid edge under the button.
struct PrimaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 15, weight: .semibold))
            .foregroundColor(.white)
            .padding(.horizontal, 28).padding(.vertical, 10)
            .background(Brand.blue)
            .cornerRadius(12)
            .shadow(color: Brand.blue.opacity(0.55), radius: 0, x: 0, y: configuration.isPressed ? 1 : 4)
            .offset(y: configuration.isPressed ? 3 : 0)
    }
}

struct Card<Content: View>: View {
    let content: Content
    init(@ViewBuilder _ content: () -> Content) { self.content = content() }
    var body: some View {
        content
            .padding(14)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color.secondary.opacity(0.10))
            .cornerRadius(12)
    }
}

struct SoonPill: View {
    let text: String
    var body: some View {
        Text(text).font(.caption)
            .padding(.horizontal, 8).padding(.vertical, 2)
            .background(Brand.dough.opacity(0.35)).cornerRadius(8)
    }
}
