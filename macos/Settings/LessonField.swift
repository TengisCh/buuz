// SPDX-License-Identifier: MIT OR Apache-2.0
// A text field that goes through the real input method and reports its marked
// text (design/macos-app/spec.md section 8.3). A SwiftUI `TextField` does not
// report marked text. The text lives only in this view. It is not logged and
// not saved.

import AppKit
import SwiftUI

final class LessonTextView: NSTextView {
    var onChange: ((LessonText) -> Void)?
    var onSubmit: (() -> Void)?
    var onFocus: ((Bool) -> Void)?
    var wantsFocus = false

    private func report() {
        onChange?(LessonText(full: string, marked: hasMarkedText() ? markedRange() : nil))
    }

    override func setMarkedText(_ string: Any, selectedRange: NSRange, replacementRange: NSRange) {
        super.setMarkedText(string, selectedRange: selectedRange, replacementRange: replacementRange)
        report()
    }

    override func unmarkText() {
        super.unmarkText()
        report()
    }

    override func insertText(_ string: Any, replacementRange: NSRange) {
        super.insertText(string, replacementRange: replacementRange)
        report()
    }

    override func didChangeText() {
        super.didChangeText()
        report()
    }

    override func doCommand(by selector: Selector) {
        // Return finishes the lesson step. It does not add a line.
        if selector == #selector(NSResponder.insertNewline(_:)) {
            onSubmit?()
            return
        }
        super.doCommand(by: selector)
        report()
    }

    override func becomeFirstResponder() -> Bool {
        let ok = super.becomeFirstResponder()
        if ok { onFocus?(true) }
        return ok
    }

    override func resignFirstResponder() -> Bool {
        let ok = super.resignFirstResponder()
        if ok { onFocus?(false) }
        return ok
    }

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        if wantsFocus, let w = window {
            DispatchQueue.main.async { w.makeFirstResponder(self) }
        }
    }

    /// Centre one line of text in the field.
    override func layout() {
        super.layout()
        let line = (font?.boundingRectForFont.height ?? 20)
        let v = max(0, (bounds.height - line) / 2)
        if abs(textContainerInset.height - v) > 0.5 { textContainerInset = NSSize(width: 18, height: v) }
    }
}

struct LessonFieldRepresentable: NSViewRepresentable {
    var fontSize: CGFloat
    var autofocus: Bool
    var onChange: (LessonText) -> Void
    var onSubmit: () -> Void
    var onFocus: (Bool) -> Void

    func makeNSView(context: Context) -> LessonTextView {
        let v = LessonTextView()
        v.isRichText = false
        v.allowsUndo = false
        v.drawsBackground = false
        v.isAutomaticQuoteSubstitutionEnabled = false
        v.isAutomaticDashSubstitutionEnabled = false
        v.isAutomaticTextReplacementEnabled = false
        v.isAutomaticSpellingCorrectionEnabled = false
        v.isContinuousSpellCheckingEnabled = false
        v.isGrammarCheckingEnabled = false
        v.isVerticallyResizable = false
        v.isHorizontallyResizable = false
        v.textContainer?.widthTracksTextView = true
        v.textContainer?.lineFragmentPadding = 0
        v.textColor = .labelColor
        v.insertionPointColor = .controlAccentColor
        v.font = .systemFont(ofSize: fontSize)
        v.wantsFocus = autofocus
        v.setAccessibilityLabel("Buuz")
        return v
    }

    func updateNSView(_ v: LessonTextView, context: Context) {
        v.onChange = onChange
        v.onSubmit = onSubmit
        v.onFocus = onFocus
        if v.font?.pointSize != fontSize { v.font = .systemFont(ofSize: fontSize) }
    }
}

/// The field with its border, halo and placeholder.
struct LessonField: View {
    var placeholder: String
    var fontSize: CGFloat = 26
    var minHeight: CGFloat = 84
    var radius: CGFloat = 14
    var autofocus = false
    /// The border for a lesson state. `nil` uses the focus colour.
    var stateColor: Color?
    var onChange: (LessonText) -> Void = { _ in }
    var onSubmit: () -> Void = {}

    @State private var empty = true
    @State private var focused = false

    var body: some View {
        let border = stateColor ?? (focused ? Tok.accent : Tok.track)
        let halo = stateColor != nil || focused
        ZStack(alignment: .leading) {
            RoundedRectangle(cornerRadius: radius, style: .continuous).fill(Tok.fill)
            LessonFieldRepresentable(
                fontSize: fontSize, autofocus: autofocus,
                onChange: { t in
                    empty = t.full.isEmpty
                    onChange(t)
                },
                onSubmit: onSubmit, onFocus: { focused = $0 })
            if empty {
                Text(placeholder).font(.system(size: fontSize)).foregroundColor(Tok.text3)
                    .padding(.leading, 18).allowsHitTesting(false).lineLimit(1)
            }
        }
        .frame(height: minHeight)
        .overlay(RoundedRectangle(cornerRadius: radius, style: .continuous).stroke(border, lineWidth: 1.5))
        .background(
            RoundedRectangle(cornerRadius: radius + 3, style: .continuous)
                .stroke(halo ? border.opacity(0.22) : .clear, lineWidth: 4).padding(-2))
        .accessibilityElement(children: .contain)
    }
}
