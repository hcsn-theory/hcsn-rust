import json
import glob
import os
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from scipy.stats import gaussian_kde

def build_phase_map(directory):
    files = glob.glob(os.path.join(directory, "particle_lifetimes_*.json"))
    if not files:
        print("No particle lifetime data found in", directory)
        return
        
    records = []
    for f in files:
        with open(f, 'r') as fp:
            data = json.load(fp)
            for k in data:
                # We need Stability and Age
                age = k["age"]
                stab = k.get("mean_stability", 0)
                coh = k.get("coherence", 0)
                mass = k.get("mass", 0)
                
                records.append({'age': age, 'stability': stab, 'coherence': coh, 'mass': mass})
                
    df = pd.DataFrame(records)
    if len(df) == 0:
        return
        
    # We define Composite Signal empirically based on our previous findings:
    # High coherence and high mass typically denote the matter phase
    df['signal'] = (df['coherence'] / df['coherence'].max()).clip(0, 1) * 0.6 + \
                   (df['mass'] / df['mass'].max()).clip(0, 1) * 0.4
                   
    # Filter the distinct phases
    vacuum = df[df['signal'] < 0.1]
    matter = df[df['signal'] > 0.3]
    
    fig, ax = plt.subplots(figsize=(10, 8))
    
    # Scatter all points
    scatter = ax.scatter(df['stability'], df['age'], c=df['signal'], 
                         cmap='RdYlGn', alpha=0.5, s=15)
    plt.colorbar(scatter, label='Composite Signal (Coherence + Mass)')
    
    # Contour for matter phase boundary
    if len(matter) > 50:
        try:
            # Scott's rule is default
            kde = gaussian_kde(matter[['stability', 'age']].values.T)
            
            s_min, s_max = df['stability'].min(), df['stability'].max()
            a_min, a_max = df['age'].min(), df['age'].max()
            
            S_grid, t_grid = np.mgrid[s_min:s_max:100j, a_min:a_max:100j]
            positions = np.vstack([S_grid.ravel(), t_grid.ravel()])
            Z = np.reshape(kde(positions).T, S_grid.shape)
            
            # Draw contour where KDE density is significant
            threshold = Z.max() * 0.1
            ax.contour(S_grid, t_grid, Z, levels=[threshold], colors='black', linewidths=2)
            ax.plot([], [], 'k-', linewidth=2, label='Matter Phase Boundary')
            ax.legend()
        except np.linalg.LinAlgError:
            print("Could not compute KDE due to singular matrix (too little variance).")
            
    ax.set_xlabel('Stability (S)')
    ax.set_ylabel('Age (t)')
    ax.set_title('Phase 2: 2D Phase Map of Emergent Matter')
    
    plt.tight_layout()
    plt.savefig('exports/phase2_map.png')
    print("Saved plot to exports/phase2_map.png")

if __name__ == "__main__":
    # Assuming baseline data from conservation test is representative
    build_phase_map("exports/conservation/patched")
