import React, { useState } from 'react';
import { Heart, Copy, Check, ExternalLink, X, ShieldCheck } from 'lucide-react';

export default function DonationModal({ isOpen, onClose, t }) {
  const [copied, setCopied] = useState(false);
  const pixKey = "24adc62c-b073-4587-974d-03fe35f6733f";

  if (!isOpen) return null;

  const handleCopy = () => {
    navigator.clipboard.writeText(pixKey);
    setCopied(true);
    setTimeout(() => setCopied(false), 3000);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-fade-in">
      <div className="relative w-full max-w-xl rounded-2xl bg-slate-900 border border-cyan-500/30 shadow-2xl p-6 text-slate-100 overflow-hidden">
        {/* Glow accent */}
        <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-cyan-500 via-blue-500 to-indigo-500"></div>

        {/* Header */}
        <div className="flex items-center justify-between pb-4 border-b border-slate-800">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-xl bg-pink-500/10 text-pink-400 border border-pink-500/20">
              <Heart className="w-6 h-6 fill-pink-500/20" />
            </div>
            <div>
              <h3 className="text-xl font-bold tracking-tight text-white">{t.donation.title}</h3>
              <p className="text-xs text-slate-400">{t.donation.subtitle}</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="mt-5 space-y-5">
          {/* PIX Box */}
          <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold uppercase tracking-wider text-cyan-400 flex items-center gap-1.5">
                🇧🇷 PIX Instantâneo
              </span>
              <span className="text-xs text-emerald-400 font-mono flex items-center gap-1">
                <ShieldCheck className="w-3.5 h-3.5" /> Chave Oficial
              </span>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="text"
                readOnly
                value={pixKey}
                className="w-full px-3 py-2 text-xs font-mono bg-slate-900 border border-slate-700 rounded-lg text-slate-200 focus:outline-none"
              />
              <button
                onClick={handleCopy}
                className="flex items-center gap-1.5 px-4 py-2 text-xs font-medium bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg transition shrink-0"
              >
                {copied ? (
                  <>
                    <Check className="w-4 h-4 text-emerald-200" />
                    <span>{t.actions.copied}</span>
                  </>
                ) : (
                  <>
                    <Copy className="w-4 h-4" />
                    <span>{t.actions.copy}</span>
                  </>
                )}
              </button>
            </div>
            {copied && (
              <p className="text-xs text-emerald-400 animate-pulse">{t.donation.pixCopied}</p>
            )}
          </div>

          {/* Wise Box */}
          <div className="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold uppercase tracking-wider text-indigo-400">
                🌐 {t.donation.wiseTitle}
              </span>
            </div>
            <div className="text-xs text-slate-300 space-y-1 font-mono">
              <p><span className="text-slate-500">Beneficiário:</span> {t.donation.wiseAccount}</p>
              <p><span className="text-slate-500">Chave / Identificador:</span> euripedesdark@gmail.com</p>
              <p className="text-slate-400 text-[11px] pt-1">Compatível com USD, EUR, GBP via transferência Wise internacional.</p>
            </div>
          </div>

          {/* GitHub Funding */}
          <div className="flex items-center justify-between p-3 rounded-xl bg-slate-950/60 border border-slate-800/80 text-xs">
            <span className="text-slate-400">GitHub Sponsors:</span>
            <a
              href="https://github.com/sponsors/euripedesdark"
              target="_blank"
              rel="noreferrer"
              className="text-cyan-400 hover:text-cyan-300 flex items-center gap-1 font-medium transition"
            >
              github.com/sponsors/euripedesdark <ExternalLink className="w-3.5 h-3.5" />
            </a>
          </div>
        </div>

        {/* Footer */}
        <div className="mt-6 pt-4 border-t border-slate-800 flex items-center justify-between">
          <span className="text-xs text-slate-500">{t.donation.thankYou}</span>
          <button
            onClick={onClose}
            className="px-4 py-2 text-xs font-medium rounded-lg bg-slate-800 hover:bg-slate-700 text-white transition"
          >
            {t.actions.close}
          </button>
        </div>
      </div>
    </div>
  );
}
